use eva::auto_impl;

use crate::{
    requests::threads::{create, delete, edit, get, search},
    service::CallStep,
};

#[auto_impl(&mut, Box)]
pub trait Threads: Send + Sync {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err>;
    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err>;

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err>;
    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err>;
    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err>;
}
