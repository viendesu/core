use std::num::NonZeroU16;

use eva::data;

#[data]
pub struct ImageInfo {
    pub format: ImageFormat,
    pub width: NonZeroU16,
    pub height: NonZeroU16,
}

#[data(copy, ord, display(name))]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Gif,
    Webp,
}

impl ImageFormat {
    pub const fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Bmp => "image/bmp",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
        }
    }
}
