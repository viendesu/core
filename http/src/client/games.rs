use super::*;

use viendesu_protocol::{
    requests::games::{create, get, search, update},
    types::game::Selector,
};

use crate::requests::games as requests;

impl Games for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(
            Method::GET,
            |get::Args {
                 game,
                 resolve_marks,
             }| {
                let path = match game {
                    Selector::Id(id) => c!("/games/{}", id.to_str()),
                    Selector::FullyQualified(fq) => c!("/games/{}/{}", fq.author, fq.slug),
                };

                (path, requests::Get { resolve_marks })
            },
        )
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(Method::POST, |args: search::Args| {
            ("/games/search".into(), args)
        })
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(Method::POST, |args: create::Args| ("/games".into(), args))
    }

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err> {
        self.do_call(Method::PATCH, |update::Args { id, update }| {
            (c!("/games/{id}"), update)
        })
    }
}
