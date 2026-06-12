service_trait! {
    pub trait Boards(viendesu_protocol::requests::boards) {
        get,

        create,
        delete,
        edit,
    }
}
