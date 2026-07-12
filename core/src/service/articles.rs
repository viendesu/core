service_trait! {
    pub trait Articles(viendesu_protocol::requests::articles) {
        get,
        search,

        create,
        delete,
        edit,
    }
}
