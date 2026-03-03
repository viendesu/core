pub mod status_code;

pub trait Request: Send + Sync + 'static {
    type Response: IsResponse;
    type Error: IsResponse + std::fmt::Display;
}

eva::trait_set! {
    pub trait IsResponse = status_code::HasStatusCode + for<'de> serde::Deserialize<'de> + serde::Serialize + Send + Sync + 'static;
}

macro_rules! impl_req {
    ($Input:ty => [$Ok:ty; $Err:ty]) => {
        const _: () = {
            use $crate::requests::Request;

            impl Request for $Input {
                type Response = $Ok;
                type Error = $Err;
            }
        };
    };
}

pub mod marks;
pub mod tabs;
pub mod users;

pub mod authors;
pub mod games;

pub mod boards;
pub mod messages;
pub mod threads;

pub mod files;
pub mod uploads;
