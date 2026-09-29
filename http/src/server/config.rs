use std::{net::SocketAddr, num::NonZeroUsize};

use eva::data;

#[data]
pub struct Http {
    pub enable: bool,
    pub listen: SocketAddr,
}

#[data]
pub struct Ssl {
    pub enable: bool,
    pub listen: SocketAddr,
}

#[data]
#[serde(default)]
pub struct Rpc {
    /// Most requests one batch may carry.
    pub max_batch: NonZeroUsize,
}

impl Default for Rpc {
    fn default() -> Self {
        Self {
            max_batch: NonZeroUsize::new(5).unwrap(),
        }
    }
}

#[data]
pub struct Config {
    pub unencrypted: Option<Http>,
    pub ssl: Option<Ssl>,
    #[serde(default)]
    pub rpc: Rpc,
}
