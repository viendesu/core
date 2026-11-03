use crate::requests::Response;
use eva::{service, str};

use crate::types::mark;

/// Game badges.
#[service(result = Response)]
pub trait Badges {
    /// Lists badges matching the query, all without one.
    fn list(
        query: Option<str::CompactString>,
    ) -> Struct! {
        badges: Vec<mark::TextEntry<mark::Badge>>,
    };

    /// Registers a badge.
    #[flags(write)]
    fn add(
        badge: str::CompactString,
    ) -> Struct! {
        id: mark::Badge,
    };
}
