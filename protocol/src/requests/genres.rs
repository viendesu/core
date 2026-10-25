use crate::requests::Response;
use eva::array;
use eva::service;

use crate::types::mark;

/// Game genres.
#[service(export, result = Response)]
pub trait Genres {
    /// Lists all genres.
    fn list() -> ok! {
        genres: array::ImmutableHeap<mark::Genre, { this![MAX] }>,
    };
    embed! {
        pub const MAX: usize = 256;
    }
}
