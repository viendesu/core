pub use Sys::healthcheck;

use crate::requests::Response;
use crate::types::True;
use eva::service;

#[service(result = Response)]
pub trait Sys {
    #[args(derive(Default))]
    #[ok(data(copy))]
    fn healthcheck() -> True;
}
