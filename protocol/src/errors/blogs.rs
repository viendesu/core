use crate::types::blog;

use eva::data;

#[data(error, display("the blog {blog} was not found"))]
pub struct NotFound {
    pub blog: blog::Selector,
}
