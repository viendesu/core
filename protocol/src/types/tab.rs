use eva::{data, str, time::Date};

use crate::types::slug;

#[str(newtype, copy)]
pub struct Id(slug::LowerSlug<23>);

#[data]
pub struct TabItem<I> {
    pub item: I,
    pub created_at: Date,
}

#[data(copy, ord)]
pub enum Kind {
    #[display("games")]
    Games,
    #[display("authors")]
    Authors,
}
