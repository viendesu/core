use crate::errors::Generic;

pub type Response<O, E> = Result<O, Generic<E>>;

pub mod badges;
pub mod genres;
pub mod tabs;
pub mod tags;

pub mod sys;

pub mod boards;
pub mod messages;
pub mod threads;

pub mod articles;
pub mod blogs;

pub mod authors;
pub mod comments;
pub mod files;
pub mod games;
pub mod uploads;
pub mod users;
