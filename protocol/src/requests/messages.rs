use crate::requests::Response;
use eva::service;

use crate::{
    errors,
    types::{True, message, thread},
};

/// Forum thread messages.
#[service(export, result = Response)]
pub trait Messages {
    /// Replaces a message's text; owner only.
    #[flags(write_op)]
    #[errors(errors::messages::NotFound)]
    fn edit(message: message::Id, text: message::Text) -> ok!(());

    /// Deletes a message.
    #[flags(write_op)]
    #[errors(errors::messages::NotFound)]
    fn delete(message: message::Id) -> ok!(True);

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
