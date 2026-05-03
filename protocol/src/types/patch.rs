use eva::data;

#[data(copy)]
#[derive(Default)]
pub enum Patch<T> {
    #[default]
    Keep,
    Change(T),
}

impl<T> Patch<T> {
    pub fn option(self) -> Option<T> {
        self.into()
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(x) => Self::Change(x),
            None => Self::Keep,
        }
    }
}

impl<T> From<Patch<T>> for Option<T> {
    fn from(value: Patch<T>) -> Self {
        match value {
            Patch::Keep => None,
            Patch::Change(v) => Some(v),
        }
    }
}
