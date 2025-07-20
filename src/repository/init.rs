use super::{Repository, Result};
use std::path::Path;

pub fn init(path: &Path) -> Result<Repository> {
    match git2::Repository::init(&path) {
        Ok(repo) => Ok(Repository { repo }),
        Err(e) => panic!("failed to init: {}", e),
    }
}
