// MIT Andrew Hickman <andrew.hickman1@sky.com>
#![allow(warnings)]
mod credentials_state;
mod error;
mod head_status;
mod origin;
mod repository;
mod repository_status;
mod settings;
mod working_tree_status;

pub use error::{Error, Result};
pub use origin::Origin;
pub use repository::{PullOutcome, Repository};
pub use settings::Settings;
