use crate::{Repository, Resolve, Result, Origin, Error};

fn authorize(repository: &Repository, origin: &Origin) -> Vec<String>{
    let token_partial = origin.token.clone().unwrap_or("".to_string());

    let token_header = format!("Authorization: token {}", token_partial);

    match &origin.token {
        None => vec![],
        Some(token) => vec![token_header]
    }
}

fn fetch<'a>(repository: &'a Repository, headers: &Vec<&str>) -> Result<git2::AnnotatedCommit<'a>>{
    let remote = repository.find_remote("origin").ok_or(Error::from_message("Remote not found"))?;

    remote.clone().fetch::<&str>(
        &[],
        Some(
            git2::FetchOptions::new()
                .custom_headers(headers)
        ),
        Some("git2kit: fetching"),
    )?;

    // this errors when bare repo is empty
    let fetch_head = repository.repo.find_reference("FETCH_HEAD")?;

    let fetch_commit = repository.repo.reference_to_annotated_commit(&fetch_head)?;

    Ok(fetch_commit)
}

fn merge(repository: &Repository, fetch_commit: git2::AnnotatedCommit) -> Result<Resolve> {
    let (merge_analysis, _) = repository.repo.merge_analysis(&[&fetch_commit])?;

    if merge_analysis.is_up_to_date() {
        log::debug!("pull: up to date");

        return Ok(Resolve { ok: true });
    } else if merge_analysis.is_fast_forward() {
        log::debug!("pull: fast forward `{}`", fetch_commit.id());

        repository.fast_forward(fetch_commit)?;

        return Ok(Resolve { ok: true });
    } else {
        // TODO diff3 resolve with resolution hunks
        // TODO add
        // TODO commit
        // TODO return conflict hunks
        return Ok(Resolve { ok: false });
    }
}

fn push(repository: &Repository, headers: &Vec<&str>) -> Result<()> {
    let remote = repository.find_remote("origin").ok_or(Error::from_message("Remote not found"))?;

    let head = repository.repo.head()?;

    remote.clone().push(&[head.name().unwrap()],
        Some(
            git2::PushOptions::new()
                .custom_headers(headers)
        ),
    )?;

    Ok(())
}

pub fn resolve(repository: &Repository, origin: &Origin) -> Result<Resolve> {
    repository.repo.remote_set_url("origin", &origin.url)?;

    let headers: Vec<String> = authorize(repository, origin);

    let headers: Vec<&str> = headers.iter().map(|s| s as &str).collect();

    match fetch(repository, &headers) {
        Ok(fetch_commit) => {
            // if fetch succeeds, try to merge
            let resolveResult = merge(repository, fetch_commit)?;

            // if merge succeeds, try to push
            push(repository, &headers)?;

            Ok(resolveResult)
        }
        Err(e) => {
            // if fetch fails, try to push
            if let Err(e) = push(repository, &headers) {
                // if both fetch and push fail, return fetch error
                return Err(e);
            }

            Ok(Resolve { ok: true })
        }
    }
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
            None,
        );

        let ours_dir = TempDir::new();

        let ours_path = ours_dir.as_ref().unwrap().path().to_path_buf();

        //let push_repository = Repository::open(&push_path)?;
        let ours_repository = Repository::clone(ours_path.clone(), &origin)?;

        // empty commit just initialize the branch
        ours_repository.commit()?;

        // push empty commit to remote
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
