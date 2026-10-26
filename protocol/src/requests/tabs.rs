use std::collections::HashMap;

use crate::requests::Response;
use eva::{True, data, int, service};

use crate::{
    errors,
    types::{author, entity, game, tab, user},
};

/// User profile tabs: named lists of games or authors.
#[service(result = Response)]
pub trait Tabs {
    /// Lists tabs of a user.
    #[errors(errors::users::NotFound)]
    fn list(
        user: user::Id,
    ) -> ok! {
        tabs: Vec<this![Tab]>,
    };
    embed! {
        #[data]
        pub enum Tab {
            Games(tab::Id),
            Authors(tab::Id),
        }
    }

    /// Lists items of a tab, paginated via `start_from`.
    #[errors(errors::tabs::InvalidItemId, errors::tabs::NoSuchTab)]
    fn list_items(
        tab: tab::Id,
        user: user::Id,
        start_from: Option<entity::Id>,
        #[serde(default)] limit: this![Limit],
        #[serde(default)] resolve_marks: bool,
    ) -> this![Value];
    embed! {
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
    #[errors(
        errors::tabs::InvalidItemId,
        errors::tabs::NoSuchItem,
        errors::tabs::NoSuchTab
    )]
    fn delete(user: user::Id, tab: tab::Id, item: entity::Id) -> True;
}
