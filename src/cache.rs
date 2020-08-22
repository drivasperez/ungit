use crate::repo::Repository;
use anyhow::Result;
use async_std::fs;
use flate2::read::GzDecoder;
use tar::Archive;

const ARCHIVE_LOCATION: &'static str = "~/.gitter_archive";

fn decompress_tarball<P: AsRef<std::path::Path>>(bytes: &[u8], path: P) -> Result<()> {
    let tar = GzDecoder::new(bytes);
    let mut archive = Archive::new(tar);
    archive.unpack(path)?;
    Ok(())
}

fn get_archive_path(repo: &str, hash: &str) -> String {
    format!("{}/{}-{}.tar.gz", ARCHIVE_LOCATION, repo, hash)
}

pub async fn save_tarball(bytes: &[u8], repo: &Repository, hash: &str) -> Result<()> {
    let repo_name = format!("{}_{}", &repo.user, &repo.repo);
    let path = get_archive_path(&repo_name, hash);
    fs::create_dir_all(ARCHIVE_LOCATION).await?;
    fs::write(path, bytes).await?;

    Ok(())
}
