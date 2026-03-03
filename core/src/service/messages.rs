use eva::auto_impl;

use crate::{
    requests::messages::{delete, edit, get, post},
    service::CallStep,
};

#[auto_impl(&mut, Box)]
pub trait Messages: Send + Sync {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err>;

    fn post(&mut self) -> impl CallStep<post::Args, Ok = post::Ok, Err = post::Err>;
    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err>;
    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err>;
}
