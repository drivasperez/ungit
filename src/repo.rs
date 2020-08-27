use crate::error::GitterError;
use anyhow::{anyhow, Context, Result};
use http_types::StatusCode;
use indicatif::ProgressBar;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Commit {
    sha: String,
    node_id: String,
}

#[derive(Serialize, Deserialize)]
struct Branch {
    name: String,
    commit: Commit,
}

#[derive(Debug)]
pub struct Repository {
    user: String,
    repo: String,
}

impl Repository {
    pub fn new(repository: String) -> Result<Self> {
        static RE: OnceCell<Regex> = OnceCell::new();

        let re = RE.get_or_init(|| Regex::new(r"^(\w+)/(\w+)$").unwrap());
        let captures = re
            .captures(&repository)
            .ok_or_else(|| anyhow!("Could not parse repository name from input"))?;

        let user = captures[1].to_owned();
        let repo = captures[2].to_owned();
        Ok(Repository { user, repo })
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn github_uri(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/branches/master",
            self.user, self.repo
        )
    }

    pub fn latest_master_tarball_uri(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/tarball/master",
            &self.user, &self.repo
        )
    }

    pub async fn fetch_latest_sha(&self) -> Result<String> {
        let uri = self.github_uri();
        let mut res = surf::get(uri).await.map_err(GitterError::NetworkError)?;

        let repo: Branch = res.body_json().await.map_err(GitterError::IOError)?;

        Ok(repo.commit.sha)
    }

    pub async fn fetch_bytes(&self) -> Result<Vec<u8>> {
        let uri = self.latest_master_tarball_uri();

        let progress = ProgressBar::new_spinner();
        progress.set_message(&format!("Downloading from {}", &uri));

        let mut res = surf::get(uri)
            .set_header("accept", "application/vnd.github.v3+json")
            .await
            .map_err(GitterError::NetworkError)?;

        let val = match res.status() {
            StatusCode::Ok => Ok(res.body_bytes().await?),
            StatusCode::Found => {
                let location = res.header("location").unwrap();
                res = surf::get(location.as_str())
                    .set_header("accept", "application/vnd.github.v3+json")
                    .await
                    .map_err(GitterError::NetworkError)
                    .context("Couldn't follow redirect")?;
                match res.status() {
                    StatusCode::Ok => Ok(res.body_bytes().await?),
                    _ => Err(GitterError::NotFound.into()),
                }
            }
            _ => Err(GitterError::NotFound.into()),
        };
        progress.finish_with_message("Download complete");
        val
    }
}

impl std::fmt::Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.repo)
    }
}
