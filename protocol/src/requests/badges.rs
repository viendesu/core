use crate::types::mark;

use eva::{data, str};

pub mod list {
    use super::*;

    #[data]
    pub struct Args {
        pub query: Option<str::CompactString>,
    }

    #[data]
    pub struct Ok {
        pub badges: Vec<mark::TextEntry<mark::Badge>>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod add {
    use super::*;

    #[data]
    pub struct Args {
        pub badge: str::CompactString,
    }

    #[data]
    pub struct Ok {
        pub id: mark::Badge,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}
