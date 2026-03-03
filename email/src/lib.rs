use eva::{data, fut::Fut};

use std::borrow::Cow;

pub mod mock;
pub mod smtp;

pub mod error;

#[data]
pub struct Letter<'a> {
    pub subject: Cow<'a, str>,
    pub contents: Cow<'a, str>,
    pub content_type: Cow<'a, str>,
}

#[eva::auto_impl(&, &mut, Arc, Box)]
pub trait Mailer: Send + Sync {
    fn send(&self, dst: &str, letter: Letter<'_>) -> impl Fut<Output = error::MailResult<()>>;
}
