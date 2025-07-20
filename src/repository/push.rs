use crate::{Repository, settings::Settings, repository_status::RepositoryStatus, Result};

pub fn push<F>(
    repository: &Repository,
    settings: &Settings,
    status: &RepositoryStatus,
    remote: Option<git2::Remote>,
) -> Result<()>
{
    //let mut remote = repository.find_remote(&remote.name.as_ref().unwrap_or(&"".to_string()))?;

    //remote.push::<String>(&[], None)?;
}
