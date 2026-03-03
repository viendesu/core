use super::*;

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .nest("/boards", boards)
        .nest("/threads", threads)
        .nest("/messages", messages)
}

fn boards<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
}

fn threads<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
}

fn messages<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
}
