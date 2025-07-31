use crate::{Repository, Result};
use std::path::Path;

// if no new files and no last commit, commit initial
// if new files and no last commit, commit initial
// if no new files and last commit, don't commit
// if new files and last commit, commit message
pub fn commit(repository: &Repository) -> Result<git2::Oid> {
    let (oid, message) = repository.add()?;

    let signature = git2::Signature::now("name", "name@mail.com")?;

    let tree = repository.repo.find_tree(oid)?;

    let oid_new = match repository.find_last_commit() {
        Ok(c) => {
            // if no new files, do not commit
            if message == "" {
                return Ok(c.id());
            };

            repository.repo.commit(
                Some("HEAD"), // point HEAD to our new commit
                &signature,   // author
                &signature,   // committer
                &message,     // commit message
                &tree,        // tree
                &[&c],        // parents
            )?
        }
        Err(_) => {
            let commit_oid = repository.repo.commit(
                None,       // point HEAD to our new commit
                &signature, // author
                &signature, // committer
                "initial",  // commit message
                &tree,      // tree
                &[],        // parents
            )?;

            let commit_new = repository.repo.find_commit(commit_oid).unwrap();

            let branch = repository.repo.branch("main", &commit_new, true).unwrap();

            let branch_ref = branch.into_reference();

            let branch_ref_name = branch_ref.name().unwrap();

            repository.repo.set_head(branch_ref_name).unwrap();

            commit_oid
        }
    };

    Ok(oid_new)
}

#[cfg(test)]
mod test {
    use crate::{Repository, Result};
    use std::fs::File;
    use std::io::prelude::*;
    use temp_dir::TempDir;

    #[test]
    fn commit_test() -> Result<()> {
        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let temp_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let temp_path = temp_dir.as_ref().unwrap().path().to_path_buf();

        let uuid = "euuid";

        let name = "etest";

        let repository = Repository::init(&temp_path)?;

        repository.commit()?;

        //let first_commit = repository.find_last_commit()?;

        //assert!(first_commit.message().unwrap() == "initial");

        //let mut file = File::create(temp_path.join("foo.txt"))?;

        //file.write_all(b"Hello, world!")?;

        //repository.commit()?;

        //let second_commit = repository.find_last_commit()?;

        //assert!(second_commit.message().unwrap() == "foo.txt");

        //assert!(first_commit.id() != second_commit.id());

        //repository.commit()?;

        //// check that does not commit when no new files
        //let third_commit = repository.find_last_commit()?;

        //assert!(third_commit.message().unwrap() == "foo.txt");

        Ok(())
    }
}
