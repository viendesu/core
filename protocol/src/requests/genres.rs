use crate::requests::Response;
use eva::{array, service};

use crate::types::mark;

/// Game genres.
#[service(result = Response)]
pub trait Genres {
    /// Lists all genres.
    fn list() -> Struct! {
        genres: array::ImmutableHeap<mark::Genre, { this::MAX }>,
    };
    embed! {
        pub const MAX: usize = 256;
    }
}
