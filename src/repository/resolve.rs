use super::fetch::fetch;
use crate::{Repository, Resolve, Result};

pub fn resolve(repository: &Repository, remote: git2::Remote) -> Result<Resolve> {
    let fetch_commit = fetch(repository, remote.clone())?;

    let (merge_analysis, _) = repository.repo.merge_analysis(&[&fetch_commit])?;

    if merge_analysis.is_up_to_date() {
        log::debug!("pull: up to date");
    } else if merge_analysis.is_fast_forward() {
        log::debug!("pull: fast forward `{}`", fetch_commit.id());

        repository.fast_forward(fetch_commit)?;
    } else {
        // TODO diff3 resolve with resolution hunks
        // TODO add
        // TODO commit
        // TODO return conflict hunks
        return Ok(Resolve { ok: false });
    }

    let head = repository.repo.head()?;

    remote.clone().push(&[head.name().unwrap()], None)?;

    Ok(Resolve { ok: true })
}

#[cfg(test)]
mod test {
    use crate::{Repository, Origin, Result};
    use temp_dir::TempDir;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::read_dir;

    #[tokio::test]
    async fn resolve_test() -> Result<()> {
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

        let temp_repository = Repository::clone(temp_path.clone(), &temp_remote)?;

        // clone the temporary directory to a pull directory
        let pull_remote = Origin::new(
            temp_path.to_str().unwrap(),
            Some("token"),
        );

        let pull_dir = TempDir::new();

        let pull_path = pull_dir.as_ref().unwrap().path().to_path_buf();

        let pull_repository = Repository::clone(pull_path.clone(), &pull_remote)?;

        // try to pull an up-to-date repository
        let outcome = pull_repository.resolve(&pull_remote)?;

        assert!(outcome == PullOutcome::UpToDate("main".to_string()));

        let mut file = File::create(temp_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!")?;

        temp_repository.commit()?;

        // try to pull a changed repository
        let outcome = pull_repository.resolve(&pull_remote)?;

        assert!(outcome == PullOutcome::FastForwarded("main".to_string()));

        // TODO check that merged foo.txt into pull_repository
        let foo = read_dir(&pull_path)?.find(|entry| {
            entry.as_ref().unwrap().file_name() == "foo.txt"
        });

        assert!(foo.is_some());

        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let origin_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let origin_path = origin_dir.as_ref().unwrap().path().to_path_buf();

        let origin_repository = Repository::init_bare(&origin_path)?;

        // clone the temporary directory to a push directory
        let origin = Origin::new(
            origin_path.to_str().unwrap(),
            Some("token"),
        );

        let push_dir = TempDir::new();

        let push_path = push_dir.as_ref().unwrap().path().to_path_buf();

        //let push_repository = Repository::open(&push_path)?;
        let push_repository = Repository::clone(push_path.clone(), &origin)?;

        push_repository.commit()?;

        let mut file = File::create(push_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!")?;

        push_repository.commit()?;

        // try to push an up-to-date repository
        push_repository.resolve(&origin)?;

        origin_repository.repo.set_head("refs/heads/main")?;

        // clone the temporary directory to a pull directory
        let pull_dir = TempDir::new();

        let pull_path = pull_dir.as_ref().unwrap().path().to_path_buf();

        Repository::clone(pull_path.clone(), &origin)?;

        // check that repo cloned
        let foo = read_dir(&pull_path)?.find(|entry| {
            entry.as_ref().unwrap().file_name() == "foo.txt"
        });

        assert!(foo.is_some());

        Ok(())
    }
}
