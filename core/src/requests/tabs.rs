use eva::{data, int};

use crate::{
    errors,
    types::{True, author, entity, game, tab, user},
};

pub mod list {
    use super::*;

    #[data]
    pub struct Args {
        pub user: user::Id,
    }

    #[data]
    pub enum Tab {
        Games(tab::Id),
        Authors(tab::Id),
    }

    #[data]
    pub struct Ok {
        pub tabs: Vec<Tab>,
    }

    #[data(error, display("_"))]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::users::NotFound),
    }
}

pub mod list_items {
    use super::*;

    #[int(u8, 1..=64)]
    pub enum Limit {}

    impl Default for Limit {
        fn default() -> Self {
            Self::POS16
        }
    }

    #[data]
    pub struct Args {
        pub tab: tab::Id,
        pub user: user::Id,
        pub start_from: Option<entity::Id>,
        #[serde(default)]
        pub limit: Limit,
        #[serde(default)]
        pub resolve_marks: bool,
    }

    #[data]
    pub enum Ok {
        Games {
            items: Vec<tab::TabItem<game::Game>>,
            #[serde(default)]
            marks: game::Marks,
        },
        Authors(Vec<tab::TabItem<author::Author>>),
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        InvalidItemId(#[from] errors::tabs::InvalidItemId),
        #[display("{_0}")]
        NoSuchTab(#[from] errors::tabs::NoSuchTab),
    }
}

pub mod insert {
    use super::*;

    #[data]
    pub struct Args {
        pub user: user::Id,
        pub tab: tab::Id,
        pub item: entity::Id,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        InvalidKind(#[from] errors::tabs::InvalidKind),
        #[display("{_0}")]
        NoSuchTab(#[from] errors::tabs::NoSuchTab),
        #[display("{_0}")]
        InvalidItemId(#[from] errors::tabs::InvalidItemId),
        #[display("{_0}")]
        Duplicate(#[from] errors::tabs::Duplicate),
    }
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub user: user::Id,
        pub tab: tab::Id,
        pub item: entity::Id,
    }

    #[data(copy)]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        InvalidItemId(#[from] errors::tabs::InvalidItemId),
        #[display("{_0}")]
        NoSuchItem(#[from] errors::tabs::NoSuchItem),
        #[display("{_0}")]
        NoSuchTab(#[from] errors::tabs::NoSuchTab),
    }
}
