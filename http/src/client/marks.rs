use super::*;

use crate::requests::marks as requests;

use viendesu_core::requests::marks::{add_badge, add_tag, list_badges, list_genres, list_tags};

impl Genres for HttpClient {
    fn list(
        &mut self,
    ) -> impl CallStep<list_genres::Args, Ok = list_genres::Ok, Err = list_genres::Err> {
        self.do_call(Method::GET, |list_genres::Args {}| {
            ("/genres".into(), requests::ListGenres {})
        })
    }
}

impl Tags for HttpClient {
    fn list(&mut self) -> impl CallStep<list_tags::Args, Ok = list_tags::Ok, Err = list_tags::Err> {
        self.do_call(Method::GET, |list_tags::Args { query }| {
            ("/tags".into(), requests::ListTags { query })
        })
    }

    fn add(&mut self) -> impl CallStep<add_tag::Args, Ok = add_tag::Ok, Err = add_tag::Err> {
        self.do_call(Method::POST, |add_tag::Args { tag }| {
            ("/tags".into(), requests::AddTag { text: tag })
        })
    }
}

impl Badges for HttpClient {
    fn list(
        &mut self,
    ) -> impl CallStep<list_badges::Args, Ok = list_badges::Ok, Err = list_badges::Err> {
        self.do_call(Method::GET, |list_badges::Args { query }| {
            ("/badges".into(), requests::ListBadges { query })
        })
    }

    fn add(&mut self) -> impl CallStep<add_badge::Args, Ok = add_badge::Ok, Err = add_badge::Err> {
        self.do_call(Method::POST, |add_badge::Args { badge }| {
            ("/badges".into(), requests::AddBadge { text: badge })
        })
    }
}
