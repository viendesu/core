use crate::requests::Response;
use eva::{array, service, str};

use crate::types::mark;

/// Game tags.
#[service(result = Response)]
pub trait Tags {
    /// Gets tags by id, at most 16; unknown ids are skipped.
    fn get_many(
        ids: array::ImmutableHeap<mark::Tag, { this::MAX }>,
    ) -> Struct! {
        tags: Vec<mark::TextEntry<mark::Tag>>,
    };
    embed! {
        pub const MAX: usize = 16;
    }

    /// Lists tags matching the query, all without one.
    fn list(
        query: Option<str::CompactString>,
    ) -> Struct! {
        tags: Vec<mark::TextEntry<mark::Tag>>,
    };

    /// Registers a tag.
    #[flags(write)]
    fn add(
        tag: str::CompactString,
    ) -> Struct! {
        id: mark::Tag,
    };
}
