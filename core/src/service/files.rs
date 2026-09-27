use eva::auto_impl;

use viendesu_protocol::requests::{Response, files::get_info};

use crate::service::CallStep;

#[auto_impl(&mut, Box)]
pub trait Files: Send + Sync {
    fn get_info(
        &mut self,
    ) -> impl CallStep<get_info::Args, Output = Response<get_info::Ok, get_info::Err>>;
}
