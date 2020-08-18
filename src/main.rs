use async_std::task;
use flate2::read::GzDecoder;
use structopt::StructOpt;
use tar::Archive;
use thiserror::Error;

#[derive(Debug, StructOpt)]
#[structopt(name = "gitter", about = "Just making a CLI")]
struct Opt {
    /// The repository to load
    #[structopt()]
    repository: String,
    #[structopt()]
    target: Option<String>,
}

#[derive(Error, Debug)]
enum GitterError {
    #[error("Could not find package")]
    NotFound,
    #[error("Network error")]
    NetworkError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Unknown status code")]
    UnknownStatusCode(u16),
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
            "https://github.com/{}/archive/master.tar.gz",
            options.repository
        );
        println!("Fetching latest from {}", uri);
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

    match res.status().as_u16() {
        200 => res.body_bytes().await.map_err(|e| GitterError::IOError(e)),
        404 => Err(GitterError::NotFound),
        302 => {
            res = surf::get(res.header("location").unwrap())
                .await
                .map_err(|e| GitterError::NetworkError(e))?;
            if res.status().as_u16() != 200 {
                Err(GitterError::TooManyRedirects)
            } else {
                res.body_bytes().await.map_err(|e| GitterError::IOError(e))
            }
        }
        n => Err(GitterError::UnknownStatusCode(n)),
    }
}
