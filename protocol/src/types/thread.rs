use eva::data;

use crate::types::{board, entity, message};

entity::define_eid! {
    pub struct Id(Thread);
}

#[data(copy)]
pub enum Selector {
    #[display("{_0}")]
    Id(#[from] Id),
}

#[data]
pub struct Thread {
    pub id: Id,
    pub by: message::By,
    pub board: board::Brief,

    pub text: message::Text,
}
