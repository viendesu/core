use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use eva::{
    collections::HashMap,
    str::{CompactString, ToCompactString},
};

use crate::{Letter, Mailer, error::MailResult};

pub type Mailbox = Vec<Letter<'static>>;
pub type Letters = HashMap<CompactString, Mailbox>;

#[derive(Debug, Clone, Default)]
pub struct Mock(Arc<Mutex<Letters>>);

impl Mock {
    pub fn collect_letters(&self) -> Letters {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

fn owned(c: Cow<'_, str>) -> Cow<'static, str> {
    Cow::Owned((*c).to_owned())
}

impl Mailer for Mock {
    async fn send(&self, dst: &str, letter: Letter<'_>) -> MailResult<()> {
        let dst = dst.to_compact_string();
        let mut this = self.0.lock().unwrap();
        let mailbox = this.entry(dst).or_default();

        mailbox.push(Letter {
            subject: owned(letter.subject),
            contents: owned(letter.contents),
            content_type: owned(letter.content_type),
        });

        Ok(())
    }
}
