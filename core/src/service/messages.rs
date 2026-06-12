service_trait! {
    pub trait Messages(viendesu_protocol::requests::messages) {
        get,

        post,
        delete,
        edit,
    }
}
