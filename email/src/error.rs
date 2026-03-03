use viendesu_core::{errors::Aux, mk_error};

pub type MailResult<O> = Result<O, MailError>;

mk_error!(MailError);

impl From<MailError> for Aux {
    fn from(value: MailError) -> Self {
        Self::Mail(eva::str::format_compact!("{}", value.0))
    }
}
