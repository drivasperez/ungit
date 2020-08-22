use anyhow::Result;
use async_std::task;
use http_types::StatusCode;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use thiserror::Error;

mod cache;

#[derive(Debug, StructOpt)]
#[structopt(name = "gitter", about = "Just making a CLI")]
struct Opt {
    /// The user
    #[structopt()]
    user: String,
    /// The repository to load
    #[structopt()]
    repository: String,
    /// The location the repository should be unpacked to
    #[structopt()]
    target: Option<String>,
}

#[derive(Error, Debug)]
enum GitterError {
    #[error("Could not find package")]
    NotFound,
    #[error("Network error")]
    NetworkError(http_types::Error),
    #[error("Unknown status code")]
    IOError(std::io::Error),
}

fn main() -> Result<()> {
    let options = Opt::from_args();
    task::block_on(async {
        let uri = format!(
            "https://codeload.github.com/{}/{}/tar.gz/master",
            options.user, options.repository
        );
        eprintln!("Fetching latest from {}", uri);
        let sha = fetch_latest_sha(&options.user, &options.repository).await?;
        eprintln!("Latest commit hash: {}", sha);
        let bytes = fetch_bytes(&uri).await?;
        cache::save_tarball(&bytes, &options.repository, &sha).await
    })
}

async fn fetch_bytes(uri: &str) -> Result<Vec<u8>> {
    let mut res = surf::get(uri)
        .await
        .map_err(|e| GitterError::NetworkError(e))?;

    eprintln!("Status code: {}", res.status());

    match res.status() {
        StatusCode::Ok => Ok(res.body_bytes().await?),
        _ => Err(GitterError::NotFound.into()),
    }
}

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

async fn fetch_latest_sha(owner: &str, repo: &str) -> Result<String> {
    let uri = format!(
        "https://api.github.com/repos/{}/{}/branches/master",
        owner, repo
    );

    let mut res = surf::get(uri)
        .await
        .map_err(|e| GitterError::NetworkError(e))?;

    let repo: Branch = res.body_json().await.map_err(|e| GitterError::IOError(e))?;

    Ok(repo.commit.sha.to_owned())
}
