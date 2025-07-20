use super::{Repository, Result};

pub fn head_branch(repository: &Repository) -> Result<git2::Branch<'_>> {
    let head = repository.repo.head()?;

    debug_assert!(head.is_branch());

    Ok(git2::Branch::wrap(head))
}
