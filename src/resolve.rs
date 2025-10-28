use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Resolve {
    pub ok: bool,
    // TODO hunks
}

impl Resolve {
    pub fn new(ok: bool) -> Self {
        Resolve { ok: true }
    }
}
