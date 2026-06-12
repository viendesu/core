service_trait! {
    pub trait Uploads(viendesu_protocol::requests::uploads) {
        list_pending,

        start,
        abort,
        finish,
    }
}
