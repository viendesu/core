use crate::requests::Response;
pub use Threads::{create, delete, edit, get};

use eva::{int, service, str};

use crate::{
    errors,
    types::{Patch, True, board, message, thread},
};

pub mod search {
    pub use super::Threads::search::*;
    use super::*;
    #[int(u8, 1..=64)]
    pub enum Limit {}
    impl Default for Limit {
        fn default() -> Self {
            Self::POS24
        }
    }
}

/// Forum threads.
#[service(result = Response)]
pub trait Threads {
    /// Lists threads, paginated via `after`.
    fn search(
        #[serde(default)] limit: crate::requests::threads::search::Limit,
        after: Option<thread::Id>,
    ) -> ok! {
        results: Vec<thread::Thread>,
    };
    /// Deletes a thread.
    #[flags(write_op)]
    #[errors(errors::threads::NotFound)]
    fn delete(thread: thread::Id) -> True;
    /// Patches a thread; owner only.
    #[flags(write_op)]
    #[args(serde_with::apply(Patch => #[serde(default)]))]
    #[errors(errors::threads::NotFound, errors::threads::NotAnOwner)]
    fn edit(thread: thread::Id, text: Patch<message::Text>) -> True;
    /// Gets a thread.
    #[errors(errors::threads::NotFound)]
    fn get(
        thread: thread::Selector,
    ) -> ok! {
        thread: thread::Thread,
    };
    /// Opens a thread on a board with its initial message.
    #[flags(write_op)]
    #[errors(errors::boards::NotFound as NoSuchBoard)]
    fn create(
        board: board::Selector,
        initial_message: message::Text,
    ) -> ok! {
        id: thread::Id,
    };
}
