use eva::data;
use serde::{Deserialize, Deserializer};

#[data(copy, not(Deserialize))]
#[derive(Default)]
pub enum Patch<T> {
    #[default]
    Keep,
    Change(T),
}

// Accept JSON null as Keep alongside "keep" and {"change": v}: clients
// express "leave untouched" as an omitted field or null interchangeably.
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Repr<T> {
            Keep,
            Change(T),
        }

        Ok(match Option::<Repr<T>>::deserialize(deserializer)? {
            None | Some(Repr::Keep) => Self::Keep,
            Some(Repr::Change(v)) => Self::Change(v),
        })
    }
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
