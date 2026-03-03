use crate::server::{
    Types,
    context::Context as Ctx,
    handler::{RouterScope, delete, get, patch, post},
};

use viendesu_core::{
    service::{CallStep, Session, SessionOf as SessionOfService},
    types::user,
};

type SessionOf<T> = Session<SessionOfService<<T as Types>::Service>>;

mod authors;
mod files;
mod games;
mod marks;
mod uploads;
mod users;

mod forum;

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .nest("/users", users::make)
        .nest("/authors", authors::make)
        .nest("/games", games::make)
        .nest("/forum", forum::make)
        .nest("/uploads", uploads::make)
        .nest("/files", files::make)
        .nest("/tags", marks::tags)
        .nest("/badges", marks::badges)
        .nest("/genres", marks::genres)
}
