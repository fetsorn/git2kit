use crate::{repository_status::RepositoryStatus, settings::Settings, Repository, Result};

pub fn push(
    repository: &Repository,
    settings: &Settings,
    status: &RepositoryStatus,
    remote: Option<git2::Remote>,
) -> Result<()> {
    let mut remote = repository.find_remote("origin").unwrap();

    let head = repository.repo.head()?;

    remote.push(&[head.name().unwrap()], None)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{Repository, Origin, Result};
    use temp_dir::TempDir;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::read_dir;

    #[tokio::test]
    async fn push_test() -> Result<()> {
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
        push_repository.push(&origin)?;

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
