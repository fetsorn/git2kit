use crate::{Repository, Result};

pub fn fetch<'a>(
    repository: &'a Repository,
    mut remote: git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>> {
    let mut connect_callbacks = git2::RemoteCallbacks::new();
    let mut fetch_callbacks = git2::RemoteCallbacks::new();
    let mut remote_connection =
        remote.connect_auth(git2::Direction::Fetch, Some(connect_callbacks), None)?;

    remote_connection.remote().fetch::<&str>(
        &[],
        Some(
            git2::FetchOptions::new()
                .remote_callbacks(fetch_callbacks)
                .download_tags(git2::AutotagOption::All)
                .update_fetchhead(true),
        ),
        Some("multi-git: fetching"),
    )?;

    let fetch_head = repository.repo.find_reference("FETCH_HEAD")?;

    Ok(repository.repo.reference_to_annotated_commit(&fetch_head)?)
}
