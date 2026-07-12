use crate::types::article;

use eva::data;

#[data(error, display("the article {article} was not found"))]
pub struct NotFound {
    pub article: article::Selector,
}

#[data(error, copy, display("you don't own the {article} article"))]
pub struct NotAnOwner {
    pub article: article::Id,
}
