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
