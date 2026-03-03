use eva::str::ToCompactString as _;

use viendesu_core::{
    errors::{Aux, Generic},
    mk_error,
};

pub type CaptchaResult<T> = Result<T, CaptchaError>;

mk_error!(CaptchaError);

impl From<CaptchaError> for Aux {
    fn from(value: CaptchaError) -> Self {
        Self::Captcha(value.0.to_compact_string())
    }
}

impl<S> From<CaptchaError> for Generic<S> {
    fn from(value: CaptchaError) -> Self {
        Self::Aux(value.into())
    }
}
