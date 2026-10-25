use crate::requests::Response;
use crate::types::mark;
use eva::service;

use eva::array;

pub mod list {
    pub use super::Genres::list::*;
    pub const MAX: usize = 256;
}

/// Game genres.
#[service(result = Response)]
pub trait Genres {
    /// Lists all genres.
    fn list() -> ok! {
        genres: array::ImmutableHeap<mark::Genre, { crate::requests::genres::list::MAX }>,
    };
}
