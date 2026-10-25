pub use Badges::{add, list};

use crate::requests::Response;
use crate::types::mark;
use eva::service;

use eva::str;

/// Game badges.
#[service(result = Response)]
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
