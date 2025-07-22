use crate::{
    credentials_state::CredentialsState, repository_status::RepositoryStatus, settings::Settings,
    Repository, Result,
};
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
    let mut remote = match remote {
        Some(remote) => remote,
        None => repository.default_remote(settings)?,
    };

    let repo_config = &repository.repo.config()?;

    let mut connect_callbacks = git2::RemoteCallbacks::new();
    let mut credentials_state = CredentialsState::default();
    connect_callbacks.credentials(move |url, username_from_url, allowed_types| {
        credentials_state.get(settings, repo_config, url, username_from_url, allowed_types)
    });

    let mut fetch_callbacks = git2::RemoteCallbacks::new();
    let mut credentials_state = CredentialsState::default();
    fetch_callbacks.credentials(move |url, username_from_url, allowed_types| {
        credentials_state.get(settings, repo_config, url, username_from_url, allowed_types)
    });

    fetch_callbacks.transfer_progress(|progress| {
        progress_callback(progress);
        true
    });

    let prune = match settings.prune {
        None => git2::FetchPrune::Unspecified,
        Some(false) => git2::FetchPrune::Off,
        Some(true) => git2::FetchPrune::On,
    };

    let mut remote_connection =
        remote.connect_auth(git2::Direction::Fetch, Some(connect_callbacks), None)?;

    let default_branch = match &status.default_branch {
        Some(name) => name.clone(),
        None => repository.default_branch_for_remote(remote_connection.remote())?,
    };
    if !status.head.on_branch(&default_branch) {
        if switch {
            if status.head.is_detached() {
                return Err(crate::Error::from_message(
                    "will not switch branch while detached",
                ));
            } else {
                repository.switch_branch(&default_branch)?;
            }
        } else {
            return Err(crate::Error::from_message("not on default branch"));
        }
    }

    remote_connection.remote().fetch::<String>(
        &[],
        Some(
            git2::FetchOptions::new()
                .remote_callbacks(fetch_callbacks)
                .download_tags(git2::AutotagOption::All)
                .update_fetchhead(true)
                .prune(prune),
        ),
        Some("multi-git: fetching"),
    )?;

    let mut fetch_head = None;
    repository
        .repo
        .fetchhead_foreach(|ref_name, remote_url, oid, is_merge| {
            if is_merge {
                fetch_head = Some(repository.repo.annotated_commit_from_fetchhead(
                    ref_name,
                    str::from_utf8(remote_url).expect("remote url is invalid utf-8"),
                    oid,
                ));
                false
            } else {
                true
            }
        })?;
    let fetch_head = match fetch_head {
        Some(fetch_head) => fetch_head?,
        None => return Err(crate::Error::from_message("no branch found to merge")),
    };

    let (merge_analysis, _) = repository.repo.merge_analysis(&[&fetch_head])?;

    if merge_analysis.is_up_to_date() {
        Ok(PullOutcome::UpToDate(default_branch))
    } else if merge_analysis.is_unborn() {
        repository.create_unborn(status, fetch_head)?;
        Ok(PullOutcome::CreatedUnborn(default_branch))
    } else if merge_analysis.is_fast_forward() {
        repository.fast_forward(fetch_head)?;
        Ok(PullOutcome::FastForwarded(default_branch))
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
        let outcome = pull_repository.pull()?;

        assert!(outcome == PullOutcome::UpToDate("main".to_string()));

        let mut file = File::create(temp_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!")?;

        temp_repository.commit()?;

        // try to push a changed repository
        let outcome = pull_repository.pull()?;

        assert!(outcome == PullOutcome::FastForwarded("main".to_string()));

        Ok(())
    }
}
