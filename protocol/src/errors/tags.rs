use eva::data;

use crate::types::mark;

#[data(error, display("no such tag: {tag}"))]
pub struct NoSuchTag {
    pub tag: mark::Tag,
}
