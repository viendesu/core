use crate::requests::Response;
use eva::service;

use crate::types::True;

#[service(export, result = Response)]
pub trait Sys {
    #[args(derive(Default))]
    fn healthcheck() -> ok!(
        #[data(copy)]
        True
    );
}
