use crate::errors::Generic;

pub type Response<O, E> = Result<O, Generic<E>>;

mod badges;
mod genres;
mod tabs;
mod tags;

mod sys;

mod boards;
mod messages;
mod threads;

mod articles;
mod blogs;

mod authors;
mod comments;
mod files;
mod games;
mod uploads;
mod users;

pub use self::{
    articles::Articles, authors::Authors, badges::Badges, blogs::Blogs, boards::Boards,
    comments::Comments, files::Files, games::Games, genres::Genres, messages::Messages, sys::Sys,
    tabs::Tabs, tags::Tags, threads::Threads, uploads::Uploads, users::Users,
};
