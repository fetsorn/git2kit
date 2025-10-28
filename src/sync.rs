use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Sync {
    pub ok: bool,
}

impl Sync {
    pub fn new(ok: bool) -> Self {
        Sync { ok: bool }
    }
}
