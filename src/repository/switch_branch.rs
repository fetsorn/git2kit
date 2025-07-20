use super::{Repository, Result};

pub fn switch_branch(repository: &Repository, branch_name: &str) -> Result<()> {
    let reference = repository
        .repo
        .find_branch(branch_name, git2::BranchType::Local)?
        .into_reference();

    repository.switch(&reference)?;

    Ok(())
}
