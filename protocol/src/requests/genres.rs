use crate::types::mark;

use eva::{array, data};

pub mod list {
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
