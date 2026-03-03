use eva::data;

use crate::types::message;

#[data(error, display("message {what} was not found"))]
pub struct NotFound {
    pub what: message::Selector,
}
