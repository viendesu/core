use std::net::SocketAddr;

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
pub struct Config {
    pub unencrypted: Option<Http>,
    pub ssl: Option<Ssl>,
}
