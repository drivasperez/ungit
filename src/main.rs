use anyhow::Result;
use async_std::task;
use structopt::StructOpt;

mod cache;
mod error;
mod repo;

use repo::Repository;

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

fn main() -> Result<()> {
    let Opt {
        user,
        repository,
        target,
    } = Opt::from_args();

    let target = target.unwrap_or(".".into());
    let repository = Repository::new(user, repository);

    task::block_on(async {
        let uri = repository.latest_master_tarball_uri();
        eprintln!("Fetching latest from {}", uri);
        let sha = repository.fetch_latest_sha().await?;
        eprintln!("Latest commit hash: {}", sha);
        if cache::check_archive_exists(&repository, &sha).await {
            eprintln!("Cached version not found");
            let bytes = repository.fetch_bytes().await?;
            cache::save_tarball(&bytes, &repository, &sha).await?;
        } else {
            eprintln!("Cached version found, unpacking");
        };
        let archive_path = cache::get_archive_path(&repository, &sha);
        eprintln!("Unpacking from {}", &archive_path);
        cache::decompress_tarball(&archive_path, &target)
    })
}
