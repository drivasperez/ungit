use crate::error::GitterError;
use anyhow::Result;
use http_types::StatusCode;
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
    pub user: String,
    pub repo: String,
}

impl Repository {
    pub fn new(user: String, repo: String) -> Self {
        Repository { user, repo }
    }

    pub fn github_uri(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/branches/master",
            self.user, self.repo
        )
    }

    pub fn latest_master_tarball_uri(&self) -> String {
        format!(
            "https://codeload.github.com/{}/{}/tar.gz/master",
            &self.user, &self.repo
        )
    }

    pub async fn fetch_latest_sha(&self) -> Result<String> {
        let uri = self.github_uri();
        let mut res = surf::get(uri)
            .await
            .map_err(|e| GitterError::NetworkError(e))?;

        let repo: Branch = res.body_json().await.map_err(|e| GitterError::IOError(e))?;

        Ok(repo.commit.sha.to_owned())
    }

    pub async fn fetch_bytes(&self) -> Result<Vec<u8>> {
        let uri = self.latest_master_tarball_uri();
        let mut res = surf::get(uri)
            .await
            .map_err(|e| GitterError::NetworkError(e))?;

        eprintln!("Status code: {}", res.status());

        match res.status() {
            StatusCode::Ok => Ok(res.body_bytes().await?),
            _ => Err(GitterError::NotFound.into()),
        }
    }
}

impl std::fmt::Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.repo)
    }
}
