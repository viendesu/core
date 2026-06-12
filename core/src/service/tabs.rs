service_trait! {
    pub trait Tabs(viendesu_protocol::requests::tabs) {
        list,
        insert,
        delete,
        list_items,
    }
}
