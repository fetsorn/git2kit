use super::{Repository, Result};
use std::path::Path;

pub fn open(path: &Path) -> Result<Repository> {
    let repo = git2::Repository::open(path)?;

    log::debug!("opened repo at `{}`", path.display());

    Ok(Repository { repo })
}
