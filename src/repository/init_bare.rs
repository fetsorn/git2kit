use super::{Repository, Result};
use std::path::Path;

pub fn init_bare(path: &Path) -> Result<Repository> {
    match git2::Repository::init_bare(&path) {
        Ok(repo) => Ok(Repository { repo }),
        Err(e) => panic!("failed to init: {}", e),
    }
}
