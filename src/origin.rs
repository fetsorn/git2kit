use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Origin {
    pub url: String,
    pub token: Option<String>,
}

impl Origin {
    pub fn new(url: &str, token: Option<&str>) -> Self {
        Origin {
            url: url.to_string(),
            token: token.map(|s| s.to_string()),
        }
    }
}

impl From<git2::Remote<'_>> for Origin {
    fn from(remote: git2::Remote) -> Origin {
        Origin {
            url: remote.url().unwrap().to_string(),
            token: None,
        }
    }
}
