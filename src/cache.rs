use crate::repo::Repository;
use anyhow::{Context, Result};
use async_std::fs;
use async_std::path::Path;
use async_std::prelude::*;
use directories::BaseDirs;
use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use std::path::PathBuf;
use tar::Archive;

const ARCHIVE_NAME: &str = ".gitter";

pub fn decompress_tarball(from: &Path, to: &Path) -> Result<()> {
    let tarball = std::fs::File::open(from)
        .with_context(|| format!("Couldn't open tarball at path {:?}", &from))?;
    let tar = GzDecoder::new(tarball);
    let mut archive = Archive::new(tar);

    let entries = archive.entries()?;

    let progress = ProgressBar::new_spinner();

    for entry in entries {
        let mut entry = entry?;
        let path = entry.path()?;
        let path = path
            .strip_prefix(path.components().next().unwrap())?
            .to_owned();

        progress.set_message(&path.to_str().unwrap_or("File"));
        entry.unpack(to.join(&path))?;
    }

    progress.finish_with_message("Finished unpacking");

    Ok(())
}

pub fn get_cache_path() -> PathBuf {
    let base_dirs = BaseDirs::new().unwrap();
    let home = base_dirs.home_dir();
    home.join(ARCHIVE_NAME)
}

pub fn get_archive_path(repo: &Repository, hash: &str) -> PathBuf {
    let cache = get_cache_path();
    let tarball_name = format!("{}_{}-{}.tar.gz", &repo.user, &repo.repo, hash);
    cache.join(tarball_name)
}

pub async fn check_archive_exists(repo: &Repository, sha: &str) -> bool {
    let path = get_archive_path(repo, sha);
    Path::exists(Path::new(&path)).await
}

pub async fn save_tarball(bytes: &[u8], repo: &Repository, hash: &str) -> Result<()> {
    let path = get_archive_path(&repo, hash);
    fs::create_dir_all(get_cache_path()).await?;
    fs::write(path, bytes).await?;

    Ok(())
}

pub async fn remove_old_version(repo: &Repository) -> Result<()> {
    let cache = get_cache_path();
    let mut files = fs::read_dir(cache).await?;

    let repo_str = format!("{}_{}", &repo.user, &repo.repo);

    while let Some(res) = files.next().await {
        let entry = res?;
        let os_str = entry.file_name();
        let file_name = os_str.to_string_lossy();
        if file_name.starts_with(&repo_str) {
            eprintln!("Removing old version");
            fs::remove_file(entry.path()).await?;
        };
    }

    Ok(())
}
