mod add;
mod clone;
mod commit;
mod create_branch;
mod create_unborn;
mod default_branch_for_remote;
mod default_remote;
mod fast_forward;
mod fetch;
mod find_last_commit;
mod head_branch;
mod head_status;
mod init;
mod init_bare;
mod open;
mod pull;
mod push;
mod status;
mod switch;
mod switch_branch;
mod try_default_branch;
mod try_open;
mod upstream_status;
mod working_tree_status;

pub use pull::PullOutcome;
pub use upstream_status::UpstreamStatus;

use super::{
    head_status::HeadStatus, origin::Origin, repository_status::RepositoryStatus,
    settings::Settings, working_tree_status::WorkingTreeStatus,
};
use crate::Result;
use std::path::{Path, PathBuf};

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

    pub fn clone(dataset_dir: PathBuf, origin: &Origin) -> Result<Self> {
        clone::clone(dataset_dir, origin)
    }

    pub fn status(&self, settings: &Settings) -> Result<(RepositoryStatus, Option<git2::Remote>)> {
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

    pub fn pull(&self, origin: &Origin) -> Result<pull::PullOutcome> {
        let settings = Settings {
            default_branch: Some("main".to_string()),
            default_remote: Some("origin".to_string()),
            ssh: None,
            editor: None,
            ignore: None,
            prune: None,
        };

        self.repo.remote_set_url("origin", &origin.url)?;

        let origin = self.find_remote("origin").unwrap();

        let (status, remote) = self.status(&settings)?;

        pull::pull(self, &settings, &status, Some(origin), true, |_| {})
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

    pub fn push(&self, origin: &Origin) -> Result<()> {
        let settings = Settings {
            default_branch: None,
            default_remote: None,
            ssh: None,
            editor: None,
            ignore: None,
            prune: None,
        };

        self.repo.remote_set_url("origin", &origin.url)?;

        let (status, remote) = self.status(&settings)?;

        push::push(self, &settings, &status, remote)
    }

    pub fn sync(&self, origin: &Origin) -> Result<Self> {
        self.repo.remote_set_url("origin", &origin.url)?;

        sync::sync(self, origin)
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

    fn find_remote(&self, remote: &str) -> Option<git2::Remote> {
        match self.repo.find_remote(remote) {
            Ok(r) => Some(r.into()),
            Err(_) => None,
        }
    }

    pub fn get_origin(&self) -> Option<Origin> {
        let origin = self.find_remote("origin")?;

        let url = origin.url()?.to_string();

        let config = self.repo.config().ok()?.snapshot().ok()?;

        let token = config.get_str("remote.origin.token").ok()?;

        match token {
            "" => Some(Origin { url, token: None }),
            _ => Some(Origin {
                url,
                token: Some(token.to_string()),
            }),
        }
    }

    pub fn set_origin(&self, origin: Origin) -> Result<()> {
        self.repo.remote_set_url("origin", &origin.url)?;

        match origin.token {
            None => (),
            Some(token) => {
                let mut config = self.repo.config()?;

                config.set_str("remote.origin.token", &token)?;
            }
        };

        Ok(())
    }

    fn remote(&self, name: &str, url: &str) -> Result<git2::Remote> {
        Ok(self.repo.remote(name, url)?)
    }
}
