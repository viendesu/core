use eva::auto_impl;

use crate::{service::AuxFut, types::session};

#[auto_impl(&mut)]
pub trait Authentication: Send + Sync {
    fn authenticate(&mut self, session: session::Token) -> impl AuxFut<()>;

    fn clear(&mut self);
}
