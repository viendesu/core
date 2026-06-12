service_trait! {
    pub trait Genres(viendesu_protocol::requests::marks) {
        list => list_genres,
    }
}

service_trait! {
    pub trait Badges(viendesu_protocol::requests::marks) {
        list => list_badges,
        add => add_badge,
    }
}

service_trait! {
    pub trait Tags(viendesu_protocol::requests::marks) {
        list => list_tags,
        add => add_tag,
    }
}
