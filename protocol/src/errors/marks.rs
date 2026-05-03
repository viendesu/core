use eva::data;

use crate::types::mark;

#[data(error, display("no such tag: {tag}"))]
pub struct NoSuchTag {
    pub tag: mark::Tag,
}

#[data(error, display("no such genre: {genre}"))]
pub struct NoSuchGenre {
    pub genre: mark::Genre,
}

#[data(error, display("no such badge: {badge}"))]
pub struct NoSuchBadge {
    pub badge: mark::Badge,
}
