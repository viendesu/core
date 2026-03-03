use super::*;

use viendesu_core::requests::boards::{create, delete, edit, get};

use crate::requests::boards as requests;

impl Boards for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, todo::<_, requests::Get>())
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(Method::POST, todo::<_, requests::Create>())
    }

    fn delete(&mut self) -> impl CallStep<delete::Args, Ok = delete::Ok, Err = delete::Err> {
        self.do_call(Method::DELETE, todo::<_, requests::Delete>())
    }

    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err> {
        self.do_call(Method::PATCH, todo::<_, requests::Edit>())
    }
}
