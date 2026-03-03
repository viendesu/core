use crate::types::mark;

use eva::{array, data, str};

#[data]
pub struct TextEntry<I> {
    pub id: I,
    pub text: str::CompactString,
}

pub mod add_tag {
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

pub mod add_badge {
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

pub mod list_genres {
    use super::*;

    pub const MAX: usize = 256;

    #[data]
    pub struct Args {}

    #[data]
    pub struct Ok {
        pub genres: array::ImmutableHeap<mark::Genre, MAX>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod list_badges {
    use super::*;

    #[data]
    pub struct Args {
        pub query: Option<str::CompactString>,
    }

    #[data]
    pub struct Ok {
        pub badges: Vec<TextEntry<mark::Badge>>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod list_tags {
    use super::*;

    #[data]
    pub struct Args {
        pub query: Option<str::CompactString>,
    }

    #[data]
    pub struct Ok {
        pub tags: Vec<TextEntry<mark::Tag>>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}
