service_trait! {
    pub trait Comments(viendesu_protocol::requests::comments) {
        list,
        replies,

        create,
        edit,
        delete,
        like,
    }
}
