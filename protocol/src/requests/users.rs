///! # Users functionality
///!
///! Subject of actions.
use eva::{data, int, str::CompactString, time::Timestamp};

use crate::{
    errors::users as errors,
    types::{True, file, patch::Patch, session, user},
};

pub mod search {
    use super::*;

    #[int(u8, 1..=64)]
    pub enum Limit {}

    impl Default for Limit {
        fn default() -> Self {
            Self::POS24
        }
    }

    #[data]
    pub struct Args {
        pub query: Option<CompactString>,
        #[serde(default)]
        pub limit: Limit,
        pub start_from: Option<user::Id>,
    }

    #[data]
    pub struct Ok {
        pub found: Vec<user::User>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod begin_auth {
    use super::*;

    #[data]
    pub struct Args {
        pub method: user::AuthoredAuth,
    }

    #[data]
    pub struct Ok {
        pub auth_session: user::AuthSessionId,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod finish_auth {
    use super::*;

    #[data]
    pub struct Args {
        pub auth_session: user::AuthSessionId,
    }

    #[data]
    pub struct Ok {
        pub session: session::Token,
        pub method: user::AuthoredAuth,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchAuthSession(#[from] errors::NoSuchAuthSession),
    }
}

pub mod check_auth {
    use super::*;

    #[data]
    pub struct Args {}

    #[data(copy)]
    pub struct Ok {
        pub user: user::Id,
        pub role: user::Role,
    }

    #[data(error, display(""))]
    pub enum Err {}
}

pub mod update {
    use super::*;

    #[data]
    pub struct Args {
        pub user: Option<user::Selector>,
        pub update: Update,
    }

    #[serde_with::apply(
        Patch => #[serde(default)]
    )]
    #[data]
    #[derive(Default)]
    pub struct Update {
        pub nickname: Patch<user::Nickname>,
        pub display_name: Patch<Option<user::DisplayName>>,
        pub bio: Patch<Option<user::Bio>>,
        pub password: Patch<user::Password>,
        pub role: Patch<user::Role>,
        pub pfp: Patch<Option<file::Id>>,
        pub email: Patch<user::Email>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::NotFound),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub user: Option<user::Selector>,
    }

    #[data]
    pub struct Ok {
        pub user: user::User,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::NotFound),
    }
}

pub mod confirm_sign_up {
    use super::*;

    #[data]
    pub struct Args {
        pub user: user::Id,
        pub token: user::SignUpCompletionToken,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        InvalidSignUpToken(#[from] errors::InvalidSignUpToken),
        #[display("{_0}")]
        NotFoundOrCompleted(#[from] errors::NotFoundOrCompleted),
    }
}

pub mod sign_in {
    use super::*;

    #[data]
    pub struct Args {
        pub nickname: user::Nickname,
        pub password: user::Password,
    }

    #[data]
    pub struct Ok {
        pub token: session::Token,
        pub expires_at: Timestamp,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::NotFound),
        #[display("{_0}")]
        InvalidPassword(#[from] errors::InvalidPassword),
        #[display("{_0}")]
        MustCompleteSignUp(#[from] errors::MustCompleteSignUp),
    }
}

pub mod sign_up {
    use super::*;

    #[data]
    pub struct Args {
        pub nickname: user::Nickname,
        pub email: user::Email,
        pub display_name: Option<user::DisplayName>,
        pub password: user::Password,
    }

    #[data]
    pub struct Ok {
        pub id: user::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        AlreadyTaken(#[from] errors::AlreadyTaken),
    }
}
