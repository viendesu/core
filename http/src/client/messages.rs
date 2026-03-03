use super::*;

use viendesu_core::requests::messages::{delete, edit, get, post};

use crate::requests::messages as requests;

impl Messages for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, todo::<_, requests::Get>())
    }

    fn post(&mut self) -> impl CallStep<post::Args, Ok = post::Ok, Err = post::Err> {
        self.do_call(Method::POST, todo::<_, requests::Post>())
    }

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err> {
        self.do_call(Method::DELETE, todo::<_, requests::Delete>())
    }

    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err> {
        self.do_call(Method::PATCH, todo::<_, requests::Edit>())
    }
}
