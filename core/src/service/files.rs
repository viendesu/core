use eva::auto_impl;

use crate::service::CallStep;
use viendesu_protocol::requests::files::get_info;

#[auto_impl(&mut, Box)]
pub trait Files {
    fn get_info(&mut self)
    -> impl CallStep<get_info::Args, Ok = get_info::Ok, Err = get_info::Err>;
}
