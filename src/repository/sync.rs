use super::fetch::fetch;
use crate::{Repository, Result, Sync};

pub fn sync(repository: &Repository, remote: Option<git2::Remote>) -> Result<()> {
    let fetch_commit = fetch(repository, remote)?;

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
        return Ok(Sync { ok: false });
    }

    // push
    let head = repository.repo.head()?;

    remote.push(&[head.name().unwrap()], None)?;

    Ok(Sync { ok: true })
}
