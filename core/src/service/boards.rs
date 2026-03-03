use eva::auto_impl;

use crate::{
    requests::boards::{create, delete, edit, get},
    service::CallStep,
};

#[auto_impl(&mut, Box)]
pub trait Boards: Send + Sync {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err>;

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err>;
    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err>;
    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err>;
}
