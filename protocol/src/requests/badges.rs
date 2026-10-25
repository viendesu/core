use crate::requests::Response;
use eva::service;
use eva::str;

use crate::types::mark;

/// Game badges.
#[service(export, result = Response)]
pub trait Badges {
    /// Lists badges matching the query, all without one.
    fn list(
        query: Option<str::CompactString>,
    ) -> ok! {
        badges: Vec<mark::TextEntry<mark::Badge>>,
    };

    /// Registers a badge.
    #[flags(write_op)]
    fn add(
        badge: str::CompactString,
    ) -> ok! {
        id: mark::Badge,
    };
}
