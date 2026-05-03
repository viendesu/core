use eva::data;

use crate::types::thread;

#[data(error, display("thread {what} was not found"))]
pub struct NotFound {
    pub what: thread::Selector,
}

#[data(error, copy, display("you don't own the {thread} thread"))]
pub struct NotAnOwner {
    pub thread: thread::Id,
}
