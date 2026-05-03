use eva::data;

use crate::types::True;

pub mod healthcheck {
    use super::*;

    #[data]
    #[derive(Default)]
    pub struct Args {}

    #[data(copy)]
    pub struct Ok(pub True);

    #[data(error, display("_"))]
    pub enum Err {}
}
