use eva::data;

use crate::types::mark;

#[data(error, display("no such genre: {genre}"))]
pub struct NoSuchGenre {
    pub genre: mark::Genre,
}
