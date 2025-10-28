use crate::{
    credentials_state::CredentialsState, repository_status::RepositoryStatus, settings::Settings,
    Repository, Result,
};
use super::fetch::fetch;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "state", content = "branch", rename_all = "snake_case")]
pub enum PullOutcome {
    UpToDate(String),
    CreatedUnborn(String),
    FastForwarded(String),
}

pub fn pull<F>(
    repository: &Repository,
    settings: &Settings,
    status: &RepositoryStatus,
    remote: Option<git2::Remote>,
    switch: bool, // whether to switch to the default branch before pulling
    mut progress_callback: F,
) -> Result<PullOutcome>
where
    F: FnMut(git2::Progress),
{
    let remote = match remote {
        Some(remote) => remote,
        None => repository.default_remote(settings)?,
    };

    let fetch_commit = fetch(repository, remote)?;

    let (merge_analysis, _) = repository.repo.merge_analysis(&[&fetch_commit])?;

    if merge_analysis.is_up_to_date() {
        log::debug!("pull: up to date");
        Ok(PullOutcome::UpToDate("main".to_owned()))
    // comment unborn or it goes off all the time
    //} else if merge_analysis.is_unborn() {
    //    repository.create_unborn(status, fetch_commit)?;
    //    Ok(PullOutcome::CreatedUnborn(default_branch))
    } else if merge_analysis.is_fast_forward() {
        log::debug!("pull: fast forward `{}`", fetch_commit.id());
        repository.fast_forward(fetch_commit)?;
        Ok(PullOutcome::FastForwarded("main".to_owned()))
    } else {
        Err(crate::Error::from_message("cannot fast-forward"))
    }
}

#[cfg(test)]
mod test {
    use crate::{Repository, Origin, Result, PullOutcome};
    use temp_dir::TempDir;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::read_dir;

    #[tokio::test]
    async fn pull_test() -> Result<()> {
        // clone the project to a temporary directory
        let pwd = std::env::current_dir()?;

        let temp_remote = Origin::new(
            pwd.to_str().unwrap(),
            Some("token"),
        );

        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let temp_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let temp_path = temp_dir.as_ref().unwrap().path().to_path_buf();

        let temp_repository = Repository::clone(temp_path.clone(), &temp_remote).await?;

        // clone the temporary directory to a pull directory
        let pull_remote = Origin::new(
            temp_path.to_str().unwrap(),
            Some("token"),
        );

        let pull_dir = TempDir::new();

        let pull_path = pull_dir.as_ref().unwrap().path().to_path_buf();

        let pull_repository = Repository::clone(pull_path.clone(), &pull_remote).await?;

        // try to pull an up-to-date repository
        let outcome = pull_repository.pull(&pull_remote)?;

        assert!(outcome == PullOutcome::UpToDate("main".to_string()));

        let mut file = File::create(temp_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!")?;

        temp_repository.commit()?;

        // try to pull a changed repository
        let outcome = pull_repository.pull(&pull_remote)?;

        assert!(outcome == PullOutcome::FastForwarded("main".to_string()));

        // TODO check that merged foo.txt into pull_repository
        let foo = read_dir(&pull_path)?.find(|entry| {
            entry.as_ref().unwrap().file_name() == "foo.txt"
        });

        assert!(foo.is_some());

        Ok(())
    }
}
