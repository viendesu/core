use eva::str;

/// String that proves successful captcha solution.
#[str(newtype)]
pub struct Token(pub str::CompactString);
