use crate::errors::Generic;

pub type Response<O, E> = Result<O, Generic<E>>;

pub mod marks;
pub mod tabs;

pub mod sys;

pub mod boards;
pub mod messages;
pub mod threads;

pub mod authors;
pub mod files;
pub mod games;
pub mod uploads;
pub mod users;
