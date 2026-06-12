service_trait! {
    pub trait Authors(viendesu_protocol::requests::authors) {
        get,
        search,

        create,
        update,
    }
}
