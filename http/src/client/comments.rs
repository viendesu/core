use super::*;

use viendesu_protocol::requests::comments::{create, delete, edit, like, list, replies};

use crate::requests::comments as requests;

impl Comments for HttpClient {
    fn list(&mut self) -> impl CallStep<list::Args, Ok = list::Ok, Err = list::Err> {
        self.do_call(
            Method::GET,
            |list::Args {
                 game,
                 sort,
                 before,
                 before_likes,
                 limit,
             }| {
                (
                    c!("/games/{game}/comments"),
                    requests::List {
                        sort,
                        before,
                        before_likes,
                        limit,
                    },
                )
            },
        )
    }

    fn replies(&mut self) -> impl CallStep<replies::Args, Ok = replies::Ok, Err = replies::Err> {
        self.do_call(
            Method::GET,
            |replies::Args {
                 comment,
                 after,
                 limit,
             }| {
                (
                    c!("/comments/{comment}/replies"),
                    requests::Replies { after, limit },
                )
            },
        )
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(Method::POST, |create::Args { game, text, parent }| {
            (
                c!("/games/{game}/comments"),
                requests::Create { text, parent },
            )
        })
    }

    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err> {
        self.do_call(Method::PATCH, |edit::Args { comment, text }| {
            (c!("/comments/{comment}"), requests::Edit { text })
        })
    }

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err> {
        self.do_call(Method::DELETE, |delete::Args { comment }| {
            (c!("/comments/{comment}"), requests::Delete {})
        })
    }

    fn like(&mut self) -> impl CallStep<like::Args, Ok = like::Ok, Err = like::Err> {
        self.do_call(Method::PUT, |like::Args { comment, liked }| {
            (c!("/comments/{comment}/like"), requests::Like { liked })
        })
    }
}
