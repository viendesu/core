use crate::types::board;

use eva::data;

#[data(error)]
pub enum AlreadyExists {
    #[display("the slug {_0} already exists")]
    Slug(#[from] board::Slug),
}

#[data(error, display("the {board} was not found"))]
pub struct NotFound {
    pub board: board::Selector,
}
