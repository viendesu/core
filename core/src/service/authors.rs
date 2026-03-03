use crate::{
    requests::authors::{create, get, search, update},
    service::CallStep,
};

use eva::auto_impl;

#[auto_impl(&mut, Box)]
pub trait Authors: Send + Sync {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err>;
    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err>;

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err>;
    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err>;
}
