use async_std::task;
use flate2::read::GzDecoder;
use http_types::StatusCode;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tar::Archive;
use thiserror::Error;

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
    UnknownStatusCode(StatusCode),
    #[error("Too many redirects")]
    TooManyRedirects,
    #[error("IO Error")]
    IOError(std::io::Error),
}

type Result<T, E = GitterError> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let options = Opt::from_args();
    task::block_on(async {
        let uri = format!(
            "https://codeload.github.com/{}/{}/tar.gz/master",
            options.user, options.repository
        );
        println!("Fetching latest from {}", uri);
        let sha = fetch_latest_sha(&options.user, &options.repository).await?;
        println!("Latest commit hash: {}", sha);
        fetch_bytes(&uri).await
    })
    .and_then(|bytes| {
        decompress_tarball(&bytes, &options.target.unwrap_or(".".into()))
            .map_err(move |e| GitterError::IOError(e))
    })
}

fn decompress_tarball<P: AsRef<std::path::Path>>(
    bytes: &[u8],
    path: P,
) -> Result<(), std::io::Error> {
    let tar = GzDecoder::new(bytes);
    let mut archive = Archive::new(tar);
    archive.unpack(path)
}

async fn fetch_bytes(uri: &str) -> Result<Vec<u8>> {
    let mut res = surf::get(uri)
        .await
        .map_err(|e| GitterError::NetworkError(e))?;

    println!("Status code: {}", res.status());

    match res.status() {
        StatusCode::Ok => res.body_bytes().await.map_err(|e| GitterError::IOError(e)),
        StatusCode::NotFound => Err(GitterError::NotFound),
        StatusCode::Found => {
            let loc = res.header("location").unwrap().as_str();
            println!("Redirecting to {}", loc);
            res = surf::get(loc)
                .await
                .map_err(|e| GitterError::NetworkError(e))?;
            if res.status() != StatusCode::Ok {
                Err(GitterError::TooManyRedirects)
            } else {
                res.body_bytes().await.map_err(|e| GitterError::IOError(e))
            }
        }
        status => Err(GitterError::UnknownStatusCode(status)),
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
