use crate::types::mark;

use eva::{array, data, str};

pub mod get_many {
    use super::*;

    pub const MAX: usize = 16;

    #[data]
    pub struct Args {
        pub ids: array::ImmutableHeap<mark::Tag, MAX>,
    }

    #[data]
    pub struct Ok {
        pub tags: Vec<mark::TextEntry<mark::Tag>>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

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
