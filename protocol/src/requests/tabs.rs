use crate::requests::Response;
use std::collections::HashMap;

pub use Tabs::{delete, insert};

use eva::{data, int, service};

use crate::{
    errors,
    types::{True, author, entity, game, tab, user},
};

pub mod list {
    pub use super::Tabs::list::*;
    use super::*;
    #[data]
    pub enum Tab {
        Games(tab::Id),
        Authors(tab::Id),
    }
}

pub mod list_items {
    pub use super::Tabs::list_items::*;
    use super::*;
    #[int(u8, 1..=64)]
    pub enum Limit {}
    impl Default for Limit {
        fn default() -> Self {
            Self::POS16
        }
    }
    #[data]
    pub enum Value {
        Games {
            items: Vec<tab::TabItem<game::Game>>,
            #[serde(default)]
            marks: game::Marks,
            authors: HashMap<author::Id, author::Mini>,
            users: HashMap<user::Id, user::Mini>,
        },
        Authors {
            items: Vec<tab::TabItem<author::Author>>,
            users: HashMap<user::Id, user::Mini>,
        },
    }
}

/// User profile tabs: named lists of games or authors.
#[service(result = Response)]
pub trait Tabs {
    /// Lists tabs of a user.
    #[errors(errors::users::NotFound)]
    fn list(
        user: user::Id,
    ) -> ok! {
        tabs: Vec<crate::requests::tabs::list::Tab>,
    };
    /// Lists items of a tab, paginated via `start_from`.
    #[errors(errors::tabs::InvalidItemId, errors::tabs::NoSuchTab)]
    fn list_items(
        tab: tab::Id,
        user: user::Id,
        start_from: Option<entity::Id>,
        #[serde(default)] limit: crate::requests::tabs::list_items::Limit,
        #[serde(default)] resolve_marks: bool,
    ) -> crate::requests::tabs::list_items::Value;
    /// Adds an item to a tab.
    #[flags(write_op)]
    #[errors(
        errors::tabs::InvalidKind,
        errors::tabs::NoSuchTab,
        errors::tabs::InvalidItemId,
        errors::tabs::Duplicate
    )]
    fn insert(user: user::Id, tab: tab::Id, item: entity::Id) -> True;
    /// Removes an item from a tab.
    #[flags(write_op)]
    #[ok(data(copy))]
    #[errors(
        errors::tabs::InvalidItemId,
        errors::tabs::NoSuchItem,
        errors::tabs::NoSuchTab
    )]
    fn delete(user: user::Id, tab: tab::Id, item: entity::Id) -> True;
}
