service_trait! {
    pub trait Threads(viendesu_protocol::requests::threads) {
        get,
        search,

        delete,
        edit,
        create,
    }
}
