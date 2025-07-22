use crate::{Repository, Result};
use std::path::Path;

pub fn add(repository: &Repository) -> Result<(git2::Oid, String)> {
    let mut index = repository.repo.index()?;

    let mut message = "".to_owned();

    let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
        let status = repository.repo.status_file(path).unwrap();

        let ret = if status.contains(git2::Status::WT_MODIFIED)
            || status.contains(git2::Status::WT_NEW)
        {
            message = if message == "" {
                format!("{}", path.display())
            } else {
                format!("{}, {}", message, path.display())
            };
            0
        } else {
            1
        };

        ret
    };

    let cb = Some(cb as &mut git2::IndexMatchedPath);

    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, cb)?;

    index.write()?;

    let oid = index.write_tree()?;

    Ok((oid, message))
}

#[cfg(test)]
mod test {
    use crate::{Repository, Result};
    use std::fs::File;
    use std::io::prelude::*;
    use temp_dir::TempDir;

    #[test]
    fn add_test() -> Result<()> {
        // create a temporary directory, will be deleted by destructor
        // must assign to keep in scope;
        let temp_dir = TempDir::new();

        // reference temp_dir to not move it out of scope
        let temp_path = temp_dir.as_ref().unwrap().path().to_path_buf();

        let repository = Repository::init(&temp_path)?;

        let mut file = File::create(temp_path.join("foo.txt"))?;

        file.write_all(b"Hello, world!")?;

        let (oid, message) = repository.add()?;

        assert!(message == "foo.txt");

        let index = repository.repo.index()?;

        let foo = index.iter().find(|e| {
            let s = std::ffi::CString::new(&e.path[..]).unwrap();

            s.to_str().unwrap() == "foo.txt"
        });

        assert!(foo.is_some());

        Ok(())
    }
}
