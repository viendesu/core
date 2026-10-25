//! # Users functionality
//!
//! Subject of actions.

use crate::requests::Response;
use eva::{data, int, service, str::CompactString, time::Timestamp};

use crate::{
    errors,
    types::{True, file, patch::Patch, session, user},
};

/// Accounts, sessions and profiles.
#[service(result = Response)]
pub trait Users {
    /// Searches users by text query, paginated via `start_from`.
    fn search(
        query: Option<CompactString>,
        #[serde(default)] limit: this![Limit],
        start_from: Option<user::Id>,
    ) -> ok! {
        found: Vec<user::User>,
    };
    embed! {
        #[int(u8, 1..=64)]
        pub enum Limit {}

        impl Default for Limit {
            fn default() -> Self {
                Self::POS24
            }
        }
    }

    /// Begins an auth session for an authored method, e.g. a Telegram bot.
    ///
    /// The returned id is a secret capability: only the caller may redeem
    /// it via `finish_auth`, and only before it expires.
    #[flags(write_op)]
    fn begin_auth(
        method: user::AuthoredAuth,
    ) -> ok! {
        auth_session: user::AuthSessionId,
    };

    /// Redeems an auth session begun by the caller for a session token.
    #[flags(write_op)]
    #[errors(errors::users::NoSuchAuthSession)]
    fn finish_auth(
        auth_session: user::AuthSessionId,
    ) -> ok! {
        session: session::Token,
        method: user::AuthoredAuth,
    };

    /// Id and role of the caller.
    fn check_auth() -> ok! {
        #![data(copy)]

        user: user::Id,
        role: user::Role,
    };

    /// Patches a user; `None` selects the caller.
    ///
    /// Updating others requires a moderation role.
    #[flags(write_op)]
    #[errors(
        errors::users::NotFound,
        FileNotFound(errors::files::NotFound),
        errors::files::UnexpectedFileClass,
        errors::users::AlreadyTaken
    )]
    fn update(user: Option<user::Selector>, update: this![Update]) -> True;
    embed! {
        #[serde_with::apply(Patch => #[serde(default)])]
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
    }

    /// Gets a user; `None` selects the caller.
    #[errors(errors::users::NotFound)]
    fn get(
        user: Option<user::Selector>,
    ) -> ok! {
        user: user::User,
    };

    /// Completes a pending sign-up with its completion token.
    #[flags(write_op)]
    #[errors(errors::users::InvalidSignUpToken, errors::users::NotFoundOrCompleted)]
    fn confirm_sign_up(user: user::Id, token: user::SignUpCompletionToken) -> True;

    /// Exchanges nickname and password for a session token.
    #[flags(write_op)]
    #[errors(
        errors::users::NotFound,
        errors::users::InvalidPassword,
        errors::users::MustCompleteSignUp
    )]
    fn sign_in(
        nickname: user::Nickname,
        password: user::Password,
    ) -> ok! {
        token: session::Token,
        expires_at: Timestamp,
    };

    /// Registers a user; signing in requires `confirm_sign_up` first.
    #[flags(write_op)]
    #[errors(errors::users::AlreadyTaken, errors::users::SignUpDisabled)]
    fn sign_up(
        nickname: user::Nickname,
        email: user::Email,
        display_name: Option<user::DisplayName>,
        password: user::Password,
    ) -> ok! {
        id: user::Id,
    };
}
