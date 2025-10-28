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
    use crate::{Repository, Origin, Result, Resolve};
    use temp_dir::TempDir;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::{read_dir, read_to_string};

    #[tokio::test]
    async fn resolve_save_test() -> Result<()> {
        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let theirs_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let theirs_path = theirs_dir.as_ref().unwrap().path().to_path_buf();

        let theirs_repository = Repository::init_bare(&theirs_path)?;

        // clone the temporary directory to a push directory
        let origin = Origin::new(
            theirs_path.to_str().unwrap(),
            Some("token"),
        );

        let ours_dir = TempDir::new();

        let ours_path = ours_dir.as_ref().unwrap().path().to_path_buf();

        //let push_repository = Repository::open(&push_path)?;
        let ours_repository = Repository::clone(ours_path.clone(), &origin)?;

        // NOTE fetching empty bare repository will error
        ours_repository.commit()?;

        ours_repository.push(&origin)?;

        // resolve an empty repository
        ours_repository.resolve(&origin)?;

        let mut file = File::create(ours_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!\n")?;

        ours_repository.commit()?;

        // resolve a fast-forward repository
        ours_repository.resolve(&origin)?;

        // at this point theirs should have foo.txt at main
        theirs_repository.repo.set_head("refs/heads/main")?;

        // clone the temporary directory to a check directory
        let check_dir = TempDir::new();

        let check_path = check_dir.as_ref().unwrap().path().to_path_buf();

        let check_repository = Repository::clone(check_path.clone(), &origin)?;

        // check that repo cloned
        let foo = read_dir(&check_path)?.find(|entry| {
            entry.as_ref().unwrap().file_name() == "foo.txt"
        });

        assert!(foo.is_some());

        file.write_all(b"foobar!\n")?;

        ours_repository.commit()?;

        ours_repository.resolve(&origin)?;

        check_repository.resolve(&origin)?;

        let contents = read_to_string(check_path.join("foo.txt"))?;

        assert!(contents == "Hello, world!\nfoobar!\n");

        Ok(())
    }
}
