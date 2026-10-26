use crate::requests::Response;
use eva::{True, service};

use crate::{
    errors,
    types::{message, thread},
};

/// Forum thread messages.
#[service(result = Response)]
pub trait Messages {
    /// Replaces a message's text; owner only.
    #[flags(write)]
    #[errors(errors::messages::NotFound)]
    fn edit(message: message::Id, text: message::Text) -> ();

    /// Deletes a message.
    #[flags(write)]
    #[errors(errors::messages::NotFound)]
    fn delete(message: message::Id) -> True;

    /// Posts a message to a thread.
    #[flags(write)]
    #[errors(NoSuchThread(errors::threads::NotFound))]
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
