service_trait! {
    pub trait Users(viendesu_protocol::requests::users) {
        get,
        check_auth,

        search,

        begin_auth,
        finish_auth,

        sign_in,
        sign_up,

        update,

        confirm_sign_up,
    }
}
