use super::*;

use crate::requests::tabs;

use viendesu_core::requests::tabs::{delete, insert, list, list_items};

impl Tabs for HttpClient {
    fn list(&mut self) -> impl CallStep<list::Args, Ok = list::Ok, Err = list::Err> {
        self.do_call(Method::GET, |list::Args { user }| {
            (c!("/users/{user}/tabs"), tabs::List {})
        })
    }

    fn list_items(
        &mut self,
    ) -> impl CallStep<list_items::Args, Ok = list_items::Ok, Err = list_items::Err> {
        self.do_call(
            Method::GET,
            |list_items::Args {
                 tab,
                 user,
                 start_from,
                 limit,
                 resolve_marks,
             }| {
                (
                    c!("/users/{user}/tabs/{tab}"),
                    tabs::ListItems {
                        resolve_marks,
                        start_from,
                        limit,
                    },
                )
            },
        )
    }

    fn insert(&mut self) -> impl CallStep<insert::Args, Ok = insert::Ok, Err = insert::Err> {
        self.do_call(Method::POST, |insert::Args { user, tab, item }| {
            (c!("/users/{user}/tabs/{tab}"), tabs::Insert { item })
        })
    }

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err> {
        self.do_call(Method::DELETE, |delete::Args { user, tab, item }| {
            (c!("/users/{user}/tabs/{tab}/{item}"), tabs::Delete {})
        })
    }
}
