use crate::requests::Response;
use crate::types::mark;

pub use Tags::{add, list};

use eva::{array, service, str};

pub mod get_many {
    pub use super::Tags::get_many::*;
    pub const MAX: usize = 16;
}

/// Game tags.
#[service(result = Response)]
pub trait Tags {
    /// Gets tags by id, at most 16; unknown ids are skipped.
    fn get_many(
        ids: array::ImmutableHeap<mark::Tag, { crate::requests::tags::get_many::MAX }>,
    ) -> ok! {
        tags: Vec<mark::TextEntry<mark::Tag>>,
    };
    /// Lists tags matching the query, all without one.
    fn list(
        query: Option<str::CompactString>,
    ) -> ok! {
        tags: Vec<mark::TextEntry<mark::Tag>>,
    };
    /// Registers a tag.
    #[flags(write_op)]
    fn add(
        tag: str::CompactString,
    ) -> ok! {
        id: mark::Tag,
    };
}
