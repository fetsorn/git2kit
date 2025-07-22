mod open;
mod add;
mod init;
mod init_bare;
mod clone;
mod status;
mod try_open;
mod head_status;
mod upstream_status;
mod working_tree_status;
mod pull;
mod push;
mod create_unborn;
mod fast_forward;
mod create_branch;
mod switch_branch;
mod switch;
mod head_branch;
mod default_remote;
mod default_branch_for_remote;
mod try_default_branch;
mod commit;
mod find_last_commit;

pub use upstream_status::UpstreamStatus;
pub use pull::PullOutcome;

use super::{head_status::HeadStatus, repository_status::RepositoryStatus, origin::Origin, working_tree_status::WorkingTreeStatus, settings::Settings};
use std::path::{Path, PathBuf};
use crate::Result;

pub struct Repository {
    repo: git2::Repository,
}

impl Repository {
    pub fn init(path: &Path) -> Result<Self> {
        init::init(path)
    }

    pub fn init_bare(path: &Path) -> Result<Self> {
        init_bare::init_bare(path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        open::open(path)
    }

    pub async fn clone(
        dataset_dir: PathBuf,
        origin: &Origin,
    ) -> Result<Self> {
        clone::clone(dataset_dir, origin).await
    }

    pub fn status(
        &self,
        settings: &Settings,
    ) -> Result<(RepositoryStatus, Option<git2::Remote>)> {
        status::status(self, settings)
    }

    pub fn try_open(path: &Path) -> Result<Option<Self>> {
        try_open::try_open(path)
    }

    fn head_status(&self) -> Result<HeadStatus> {
        head_status::head_status(self)
    }

    fn upstream_status(&self, head_status: &HeadStatus) -> Result<upstream_status::UpstreamStatus> {
        upstream_status::upstream_status(self, head_status)
    }

    pub fn working_tree_status(&self) -> Result<WorkingTreeStatus> {
        working_tree_status::working_tree_status(self)
    }

    pub fn pull(&self) -> Result<pull::PullOutcome> {
        let settings = Settings {
            default_branch: None,
            default_remote: None,
            ssh: None,
            editor: None,
            ignore: None,
            prune: None,
        };

        let (status, remote) = self.status(&settings)?;

        pull::pull(self, &settings, &status, remote, true, |_| {})
    }

    fn create_unborn(
        &self,
        status: &RepositoryStatus,
        fetch_commit: git2::AnnotatedCommit,
    ) -> Result<()> {
        create_unborn::create_unborn(self, status, fetch_commit)
    }

    fn fast_forward(&self, fetch_commit: git2::AnnotatedCommit) -> Result<()> {
        fast_forward::fast_forward(self, fetch_commit)
    }

    pub fn push(&self) -> Result<()> {
        let settings = Settings {
            default_branch: None,
            default_remote: None,
            ssh: None,
            editor: None,
            ignore: None,
            prune: None,
        };

        let (status, remote) = self.status(&settings)?;

        push::push(self, &settings, &status, remote)
    }

    fn add(&self) -> Result<(git2::Oid, String)> {
        add::add(self)
    }

    fn create_branch(&self, settings: &Settings, name: &str) -> Result<()> {
        create_branch::create_branch(self, settings, name)
    }

    fn switch_branch(&self, branch_name: &str) -> Result<()> {
        switch_branch::switch_branch(self, branch_name)
    }

    fn switch(&self, reference: &git2::Reference) -> Result<()> {
        switch::switch(self, reference)
    }

    fn head_branch(&self) -> Result<git2::Branch<'_>> {
        head_branch::head_branch(self)
    }

    fn default_remote(&self, settings: &Settings) -> Result<git2::Remote> {
        default_remote::default_remote(self, settings)
    }

    fn default_branch_for_remote(&self, remote: &git2::Remote) -> Result<String> {
        default_branch_for_remote::default_branch_for_remote(self, remote)
    }

    fn try_default_branch(&self, settings: &Settings) -> (Option<String>, Option<git2::Remote>) {
        try_default_branch::try_default_branch(self, settings)
    }

    pub fn commit(&self) -> Result<git2::Oid> {
        commit::commit(self)
    }

    fn find_last_commit(&self) -> Result<git2::Commit> {
        find_last_commit::find_last_commit(self)
    }

    fn find_remote(&self, remote: &str) -> Result<git2::Remote> {
        Ok(self.repo.find_remote(remote)?)
    }

    pub fn get_origin(&self) -> Result<Origin> {
        let origin = self.find_remote("origin")?;

        Ok(origin.into())
    }

    pub fn set_origin(&self, origin: Origin) -> Result<()> {
        self.repo.remote_set_url("origin", &origin.url)?;

        Ok(())
    }

    fn remote(&self, name: &str, url: &str) -> Result<git2::Remote> {
        Ok(self.repo.remote(name, url)?)
    }
}
