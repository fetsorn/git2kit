use super::{Repository, Result};

pub fn find_last_commit(repository: &Repository) -> Result<git2::Commit> {
    let obj = repository
        .repo
        .head()?
        .resolve()?
        .peel(git2::ObjectType::Commit)?;

    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit").into())
}
