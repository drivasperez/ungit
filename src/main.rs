use anyhow::Result;
use async_std::path::Path;
use async_std::task;
use structopt::StructOpt;

mod cache;
mod error;
mod repo;

use repo::Repository;

#[derive(Debug, StructOpt)]
#[structopt(name = "gitter", about = "Just making a CLI")]
struct Opt {
    /// The repository to load
    #[structopt()]
    repo: String,
    /// The location the repository should be unpacked to
    #[structopt()]
    target: Option<String>,
}

fn main() -> Result<()> {
    let Opt { repo, target } = Opt::from_args();

    let repository = Repository::new(repo)?;
    let target = target.unwrap_or_else(|| repository.repo().to_owned());

    task::block_on(async {
        eprintln!("Fetching latest...");
        let sha = repository.fetch_latest_sha().await?;
        if !cache::check_archive_exists(&repository, &sha).await {
            cache::remove_old_version(&repository).await?;
            let bytes = repository.fetch_bytes().await?;
            cache::save_tarball(&bytes, &repository, &sha).await?;
        } else {
            eprintln!("Cached version found, unpacking");
        };
        let archive_path = cache::get_archive_path(&repository, &sha);
        cache::decompress_tarball(Path::new(&archive_path), Path::new(&target))
    })
}
