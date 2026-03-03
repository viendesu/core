use super::*;

use viendesu_core::requests::authors::{create, get, search, update};

use crate::requests::authors as requests;

impl Authors for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, |get::Args { author }| {
            (c!("/authors/{author}"), requests::Get {})
        })
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(
            Method::GET,
            |search::Args {
                 query,
                 owned_by,
                 start_from,
                 limit,
             }| {
                (
                    c!("/authors"),
                    requests::Search {
                        query,
                        start_from,
                        owned_by,
                        limit,
                    },
                )
            },
        )
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(
            Method::POST,
            |create::Args {
                 title,
                 slug,
                 pfp,
                 description,
                 owner,
             }| {
                (
                    c!("/authors"),
                    requests::Create {
                        title,
                        description,
                        slug,
                        pfp,
                        owner,
                    },
                )
            },
        )
    }

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err> {
        self.do_call(Method::PATCH, |update::Args { author, update }| {
            (
                c!("/authors/{author}"),
                requests::Update {
                    title: update.title,
                    description: update.description,
                    pfp: update.pfp,
                    slug: update.slug,
                    verified: update.verified,
                },
            )
        })
    }
}
