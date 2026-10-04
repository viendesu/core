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
        pub tags: Vec<mark::TextEntry<mark::Tag>>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod add {
    use super::*;

    #[data]
    pub struct Args {
        pub tag: str::CompactString,
    }

    #[data]
    pub struct Ok {
        pub id: mark::Tag,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}
