//! OpenAPI 3.1 document generation from the typed route table.
//!
//! Every [`RouterScope::route`](super::handler::RouterScope::route) call
//! contributes one operation: the request type describes the input (query
//! string for GET, body otherwise), `ConstStatusCode` gives the success
//! status, and `ErrorVariants` gives the per-variant error statuses.
//!
//! [`schemars`] emits JSON Schema 2020-12, which OpenAPI 3.1 uses natively.

use std::collections::BTreeMap;

use http::status::StatusCode;
use schemars::generate::{SchemaGenerator, SchemaSettings};
use serde_json::{Map, Value, json};

use viendesu_protocol::errors::Aux;

use crate::{
    format::Format,
    requests::{
        InputSchema, Request,
        status_code::{ConstStatusCode, ErrorVariants},
    },
    server::handler::Verb,
};

pub struct Collector {
    generator: SchemaGenerator,
    paths: BTreeMap<String, Map<String, Value>>,
}

pub fn join_path(prefix: &str, path: &str) -> String {
    if path == "/" {
        if prefix.is_empty() {
            "/".to_owned()
        } else {
            prefix.to_owned()
        }
    } else {
        format!("{prefix}{path}")
    }
}

impl Collector {
    pub fn new() -> Self {
        let settings = SchemaSettings::draft2020_12()
            .with(|s| s.definitions_path = "/components/schemas".to_owned());

        Self {
            generator: settings.into_generator(),
            paths: BTreeMap::new(),
        }
    }

    pub fn operation<R: Request>(&mut self, verb: Verb, path: String) {
        let mut operation = Map::new();
        let mut parameters = path_parameters(&path);

        if verb == Verb::Get {
            parameters.extend(query_parameters::<R>());
        } else {
            let body = match R::input_schema(&mut self.generator) {
                InputSchema::Data(schema) => json!({
                    "required": true,
                    "content": { Format::Json.mime_type(): { "schema": schema } },
                }),
                InputSchema::Multipart => json!({
                    "required": true,
                    "content": {
                        "multipart/form-data": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "file": { "type": "string", "format": "binary" },
                                },
                                "required": ["file"],
                            },
                        },
                    },
                }),
            };
            operation.insert("requestBody".to_owned(), body);
        }

        if !parameters.is_empty() {
            operation.insert("parameters".to_owned(), Value::Array(parameters));
        }
        operation.insert("responses".to_owned(), self.responses::<R>());

        let displaced = self
            .paths
            .entry(path)
            .or_default()
            .insert(verb.name().to_owned(), Value::Object(operation));
        debug_assert!(displaced.is_none(), "duplicate (path, method) route");
    }

    fn responses<R: Request>(&mut self) -> Value {
        let mut responses = Map::new();

        let ok_schema = self.generator.subschema_for::<R::Response>();
        responses.insert(
            <R::Response as ConstStatusCode>::STATUS.as_str().to_owned(),
            json!({
                "description": "success",
                "content": { Format::Json.mime_type(): { "schema": {
                    "type": "object",
                    "properties": { "ok": ok_schema },
                    "required": ["ok"],
                }}},
            }),
        );

        // Per status: names for the response description + `error` field schemas.
        let mut errors = BTreeMap::<u16, (Vec<String>, Vec<Value>)>::new();

        for variant in R::Error::error_variants(&mut self.generator) {
            let (names, schemas) = errors.entry(variant.status.as_u16()).or_default();

            // Specific errors are serialized externally tagged: `{"<name>": <payload>}`.
            let mut properties = Map::new();
            properties.insert(
                variant.name.clone(),
                serde_json::to_value(variant.schema).unwrap(),
            );
            schemas.push(json!({
                "type": "object",
                "properties": properties,
                "required": [variant.name],
            }));
            names.push(format!("`{}`", variant.name));
        }

        // Failures of auxiliary systems (authentication, malformed input,
        // internals) are possible on every endpoint; the status list mirrors
        // `HasStatusCode for Aux` in `requests::status_code`.
        let aux_schema = serde_json::to_value(self.generator.subschema_for::<Aux>()).unwrap();
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            let (names, schemas) = errors.entry(status.as_u16()).or_default();
            schemas.push(aux_schema.clone());
            names.push("auxiliary failure".to_owned());
        }

        for (status, (names, mut schemas)) in errors {
            let error_schema = if schemas.len() == 1 {
                schemas.pop().unwrap()
            } else {
                json!({ "anyOf": schemas })
            };

            responses.insert(
                status.to_string(),
                json!({
                    "description": names.join(", "),
                    "content": { Format::Json.mime_type(): { "schema": {
                        "type": "object",
                        "properties": {
                            "error": error_schema,
                            "description": { "type": "string" },
                        },
                        "required": ["error", "description"],
                    }}},
                }),
            );
        }

        Value::Object(responses)
    }

    pub fn into_document(mut self) -> Value {
        let paths: Map<String, Value> = self
            .paths
            .into_iter()
            .map(|(path, operations)| (path, Value::Object(operations)))
            .collect();

        json!({
            "openapi": "3.1.0",
            "info": {
                "title": "VienDesu API",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "paths": paths,
            "components": { "schemas": self.generator.take_definitions() },
        })
    }
}

fn path_parameters(path: &str) -> Vec<Value> {
    // Path parameters are deserialized from string segments inside handlers
    // (via `Context::path`), so their wire type is always a string.
    let mut parameters = Vec::new();
    let mut rest = path;

    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('}') else { break };

        parameters.push(json!({
            "name": &rest[..end],
            "in": "path",
            "required": true,
            "schema": { "type": "string" },
        }));
        rest = &rest[end + 1..];
    }

    parameters
}

#[cfg(test)]
mod tests {
    use super::*;

    use viendesu_protocol::requests::users::{search, sign_in};

    #[test]
    fn body_request_operation() {
        let mut collector = Collector::new();
        collector.operation::<sign_in::Args>(Verb::Post, "/users/sign_in".to_owned());
        let document = collector.into_document();

        let operation = &document["paths"]["/users/sign_in"]["post"];
        assert!(operation["requestBody"]["content"]["application/json"]["schema"].is_object());

        let responses = operation["responses"].as_object().unwrap();
        // success + 400/401/403/404/500: NotFound => 404,
        // InvalidPassword and MustCompleteSignUp => 403 (merged with aux).
        assert_eq!(
            responses.keys().collect::<Vec<_>>(),
            ["200", "400", "401", "403", "404", "500"]
        );

        let not_found =
            &responses["404"]["content"]["application/json"]["schema"]["properties"]["error"];
        assert_eq!(
            not_found["properties"]["not_found"]["$ref"],
            "#/components/schemas/NotFound"
        );

        let forbidden =
            &responses["403"]["content"]["application/json"]["schema"]["properties"]["error"];
        assert_eq!(forbidden["anyOf"].as_array().unwrap().len(), 3);

        assert!(document["components"]["schemas"]["NotFound"].is_object());
    }

    #[test]
    fn query_request_operation() {
        let mut collector = Collector::new();
        collector.operation::<search::Args>(Verb::Get, "/users".to_owned());
        let document = collector.into_document();

        let operation = &document["paths"]["/users"]["get"];
        assert!(operation.get("requestBody").is_none());

        let parameters = operation["parameters"].as_array().unwrap();
        let mut names: Vec<_> = parameters
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["limit", "query", "start_from"]);

        // Query parameter schemas are inlined, not referenced.
        assert!(
            parameters
                .iter()
                .all(|p| p["schema"].get("$ref").is_none())
        );
    }

    #[test]
    fn path_parameters_from_template() {
        let parameters = path_parameters("/users/{user}/confirm/{token}");
        assert_eq!(parameters[0]["name"], "user");
        assert_eq!(parameters[1]["name"], "token");
        assert_eq!(parameters.len(), 2);
    }

    #[test]
    fn join_paths() {
        assert_eq!(join_path("", "/"), "/");
        assert_eq!(join_path("/users", "/"), "/users");
        assert_eq!(join_path("/users/{user}/tabs", "/{tab}"), "/users/{user}/tabs/{tab}");
    }
}

fn query_parameters<R: Request>() -> Vec<Value> {
    // A separate inlining generator: query argument schemas are decomposed
    // into parameters right here and must not leak refs into components.
    let mut generator = SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .into_generator();

    let InputSchema::Data(schema) = R::input_schema(&mut generator) else {
        return Vec::new();
    };

    let schema = serde_json::to_value(schema).unwrap();
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };

    properties
        .iter()
        .map(|(name, schema)| {
            json!({
                "name": name,
                "in": "query",
                "required": required.contains(&name.as_str()),
                "schema": schema,
            })
        })
        .collect()
}
