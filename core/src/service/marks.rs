use eva::auto_impl;

use crate::{
    requests::marks::{add_badge, add_tag, list_badges, list_genres, list_tags},
    service::CallStep,
};

#[auto_impl(&mut)]
pub trait Genres: Send + Sync {
    fn list(
        &mut self,
    ) -> impl CallStep<list_genres::Args, Ok = list_genres::Ok, Err = list_genres::Err>;
}

#[auto_impl(&mut)]
pub trait Badges: Send + Sync {
    fn list(
        &mut self,
    ) -> impl CallStep<list_badges::Args, Ok = list_badges::Ok, Err = list_badges::Err>;
    fn add(&mut self) -> impl CallStep<add_badge::Args, Ok = add_badge::Ok, Err = add_badge::Err>;
}

#[auto_impl(&mut)]
pub trait Tags: Send + Sync {
    fn list(&mut self) -> impl CallStep<list_tags::Args, Ok = list_tags::Ok, Err = list_tags::Err>;
    fn add(&mut self) -> impl CallStep<add_tag::Args, Ok = add_tag::Ok, Err = add_tag::Err>;
}
