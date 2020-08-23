use crate::repo::Repository;
use anyhow::{Context, Result};
use async_std::fs;
use async_std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;

const ARCHIVE_LOCATION: &'static str = "./gitter_archive";

pub fn decompress_tarball(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
    let tarball = std::fs::File::open(from)
        .with_context(|| format!("Couldn't open tarball at path {:?}", &from))?;
    let tar = GzDecoder::new(tarball);
    let mut archive = Archive::new(tar);
    archive.unpack(to).context("Co")?;
    Ok(())
}

pub fn get_archive_path(repo: &Repository, hash: &str) -> String {
    let repo_name = format!("{}_{}", &repo.user, &repo.repo);
    format!("{}/{}-{}.tar.gz", ARCHIVE_LOCATION, repo_name, hash)
}

pub async fn check_archive_exists(repo: &Repository, sha: &str) -> bool {
    let path = get_archive_path(repo, sha);
    Path::exists(Path::new(&path)).await
}

pub async fn save_tarball(bytes: &[u8], repo: &Repository, hash: &str) -> Result<()> {
    let path = get_archive_path(&repo, hash);
    fs::create_dir_all(ARCHIVE_LOCATION).await?;
    fs::write(path, bytes).await?;

    Ok(())
}
