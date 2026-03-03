use eva::auto_impl;

use crate::{
    requests::tabs::{delete, insert, list, list_items},
    service::CallStep,
};

#[auto_impl(&mut)]
pub trait Tabs: Send + Sync {
    fn list(&mut self) -> impl CallStep<list::Args, Ok = list::Ok, Err = list::Err>;
    fn insert(&mut self) -> impl CallStep<insert::Args, Ok = insert::Ok, Err = insert::Err>;
    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err>;
    fn list_items(
        &mut self,
    ) -> impl CallStep<list_items::Args, Ok = list_items::Ok, Err = list_items::Err>;
}
