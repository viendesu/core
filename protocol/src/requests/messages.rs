pub use Messages::{delete, edit, get, post};

use crate::requests::Response;
use crate::{
    errors,
    types::{True, message, thread},
};
use eva::service;

/// Forum thread messages.
#[service(result = Response)]
pub trait Messages {
    /// Replaces a message's text; owner only.
    #[flags(write_op)]
    #[errors(errors::messages::NotFound)]
    fn edit(message: message::Id, text: message::Text) -> ();
    /// Deletes a message.
    #[flags(write_op)]
    #[errors(errors::messages::NotFound)]
    fn delete(message: message::Id) -> True;
    /// Posts a message to a thread.
    #[flags(write_op)]
    #[errors(errors::threads::NotFound as NoSuchThread)]
    fn post(
        thread: thread::Selector,
        text: message::Text,
    ) -> ok! {
        id: message::Id,
    };
    /// Gets a message.
    #[errors(errors::messages::NotFound)]
    fn get(
        message: message::Selector,
    ) -> ok! {
        message: message::Message,
    };
}
