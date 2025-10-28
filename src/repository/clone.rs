use super::Repository;
use crate::{Origin, Result};
use std::path::PathBuf;

pub fn clone(
    dataset_dir: PathBuf,
    remote: &Origin,
) -> Result<Repository> {
    // clone to dataset_dir from remote_url with remote_token
    // let repo = match Repository::clone(remote.url, dataset_dir) {
    //     Ok(repo) => repo,
    //     Err(e) => panic!("failed to clone: {}", e),
    // };

    // Prepare callbacks.
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        git2::Cred::username(remote.token.as_ref().unwrap_or(&"".to_string()))
    });

    // Prepare fetch options.
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    // Prepare builder.
    let mut builder = git2::build::RepoBuilder::new();

    // only works if origin has main
    //builder.branch("main");

    builder.fetch_options(fo);

    // Clone the project.
    let repo = builder.clone(
        &remote.url,
        &dataset_dir,
    )?;

    // TODO rename default branch to main

    // set config.remote.origin.url

    // set config.remote.origin.token

    Ok(Repository {repo})
}

#[cfg(test)]
mod test {
    use super::{Repository, Origin, Result};
    use std::fs::read_dir;
    use temp_dir::TempDir;

    #[tokio::test]
    async fn clone_test() -> Result<()> {
        // clone the project to a temporary directory
        let pwd = std::env::current_dir()?;

        let remote = Origin::new(
            pwd.to_str().unwrap(),
            Some("token"),
        );

        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let temp_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let temp_path = temp_dir.as_ref().unwrap().path().to_path_buf();


        let repository = Repository::clone(temp_path.clone(), &remote)?;

        assert!(repository.repo.path() == temp_path.join(".git"));

        // check that repo cloned
        let gitrepo = read_dir(&temp_path)?.find(|entry| {
            entry.as_ref().unwrap().file_name() == ".git"
        });

        assert!(gitrepo.is_some());

        Ok(())
    }
}
