pub mod status_code;

use schemars::{JsonSchema, Schema, SchemaGenerator};

pub trait Request: Send + Sync + 'static + DescribeInput {
    type Response: IsResponse + JsonSchema + status_code::ConstStatusCode;
    type Error: IsResponse + JsonSchema + status_code::ErrorVariants + std::fmt::Display;
}

/// How request arguments enter the endpoint on the wire.
pub enum InputSchema {
    /// Structured data: query string for GET, body otherwise.
    Data(Schema),
    /// Raw multipart upload, not described by a schema.
    Multipart,
}

pub trait DescribeInput {
    fn input_schema(generator: &mut SchemaGenerator) -> InputSchema;
}

eva::trait_set! {
    pub trait IsResponse = status_code::HasStatusCode + for<'de> serde::Deserialize<'de> + serde::Serialize + Send + Sync + 'static;
}

macro_rules! impl_req {
    ($Input:ty => [$Ok:ty; $Err:ty]) => {
        impl_req!(@request $Input => [$Ok; $Err]);

        const _: () = {
            use $crate::requests::{DescribeInput, InputSchema};

            impl DescribeInput for $Input {
                fn input_schema(
                    generator: &mut ::schemars::SchemaGenerator,
                ) -> InputSchema {
                    InputSchema::Data(generator.subschema_for::<Self>())
                }
            }
        };
    };
    ($Input:ty => multipart [$Ok:ty; $Err:ty]) => {
        impl_req!(@request $Input => [$Ok; $Err]);

        const _: () = {
            use $crate::requests::{DescribeInput, InputSchema};

            impl DescribeInput for $Input {
                fn input_schema(_: &mut ::schemars::SchemaGenerator) -> InputSchema {
                    InputSchema::Multipart
                }
            }
        };
    };
    (@request $Input:ty => [$Ok:ty; $Err:ty]) => {
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

pub mod articles;
pub mod blogs;

pub mod files;
pub mod uploads;
