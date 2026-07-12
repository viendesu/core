use crate::types::blog;

use eva::data;

#[data(error, copy, display("the blog {blog} was not found"))]
pub struct NotFound {
    pub blog: blog::Id,
}

#[data(error, copy, display("you don't own the {blog} blog"))]
pub struct NotAnOwner {
    pub blog: blog::Id,
}
