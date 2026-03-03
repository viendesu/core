use eva::auto_impl;

use crate::{
    requests::uploads::{abort, finish, list_pending, start},
    service::CallStep,
};

#[auto_impl(&mut, Box)]
pub trait Uploads: Send + Sync {
    fn list_pending(
        &mut self,
    ) -> impl CallStep<list_pending::Args, Ok = list_pending::Ok, Err = list_pending::Err>;

    fn start(&mut self) -> impl CallStep<start::Args, Ok = start::Ok, Err = start::Err>;
    fn abort(&mut self) -> impl CallStep<abort::Args, Ok = abort::Ok, Err = abort::Err>;
    fn finish(&mut self) -> impl CallStep<finish::Args, Ok = finish::Ok, Err = finish::Err>;
}
