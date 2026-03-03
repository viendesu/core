use super::*;

use viendesu_core::requests::threads::{create, delete, edit, get, search};

use crate::requests::threads as requests;

impl Threads for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, todo::<_, requests::Get>())
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(Method::POST, todo::<_, requests::Search>())
    }

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err> {
        self.do_call(Method::DELETE, todo::<_, requests::Delete>())
    }

    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err> {
        self.do_call(Method::PATCH, todo::<_, requests::Edit>())
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(Method::POST, todo::<_, requests::Create>())
    }
}
