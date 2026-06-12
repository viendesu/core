service_trait! {
    pub trait Games(viendesu_protocol::requests::games) {
        get,
        search,

        create,
        update,
    }
}
