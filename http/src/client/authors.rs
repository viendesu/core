use super::*;

use viendesu_protocol::requests::authors::{create, get, search, update};

use crate::requests::authors as requests;

impl Authors for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, |get::Args { author }| {
            (c!("/authors/{author}"), requests::Get {})
        })
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(Method::GET, |args: search::Args| (c!("/authors"), args))
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(Method::POST, |args: create::Args| (c!("/authors"), args))
    }

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err> {
        self.do_call(Method::PATCH, |update::Args { author, update }| {
            (c!("/authors/{author}"), update)
        })
    }
}
