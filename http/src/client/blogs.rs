use super::*;

use viendesu_protocol::requests::blogs::{edit, get};

use crate::requests::blogs as requests;

impl Blogs for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, todo::<_, requests::Get>())
    }

    fn edit(&mut self) -> impl CallStep<edit::Args, Ok = edit::Ok, Err = edit::Err> {
        self.do_call(Method::PATCH, todo::<_, requests::Edit>())
    }
}
