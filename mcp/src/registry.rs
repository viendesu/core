use std::{future::Future, pin::Pin};

use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use viendesu_core::service::{IsSession, Session};
use viendesu_protocol::requests::Response;

pub type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Result of a single tool invocation.
pub enum CallOutcome {
    /// Successful call, carries the serialized `Ok` payload.
    Ok(Value),
    /// Domain or auxiliary error, carries the serialized error payload.
    ToolError(Value),
    /// Arguments did not match the tool's input schema.
    InvalidArgs(String),
}

type Run<S> = Box<dyn Fn(Session<S>, Value) -> BoxFut<CallOutcome> + Send + Sync>;

struct Entry<S> {
    name: &'static str,
    spec: Value,
    run: Run<S>,
}

/// Ordered registry of MCP tools over sessions of `S`.
pub struct Tools<S> {
    entries: Vec<Entry<S>>,
}

impl<S> Default for Tools<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Tools<S> {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Tool specifications as expected by `tools/list`.
    pub fn list(&self) -> Value {
        Value::Array(self.entries.iter().map(|e| e.spec.clone()).collect())
    }

    /// Concatenates two registries. Panics on duplicate tool names.
    pub fn merge(mut self, other: Self) -> Self {
        for entry in other.entries {
            self.assert_vacant(entry.name);
            self.entries.push(entry);
        }
        self
    }

    /// Invokes tool `name` with JSON `args`, `None` if there is no such tool.
    pub fn call(&self, name: &str, session: Session<S>, args: Value) -> Option<BoxFut<CallOutcome>> {
        let entry = self.entries.iter().find(|e| e.name == name)?;
        Some((entry.run)(session, args))
    }

    fn assert_vacant(&self, name: &str) {
        assert!(
            self.entries.iter().all(|e| e.name != name),
            "duplicate tool {name:?}"
        );
    }
}

impl<S: IsSession + 'static> Tools<S> {
    /// Registers a tool backed by a service call.
    ///
    /// Input and output schemas are derived from `Args` and `O`; domain
    /// errors are serialized and reported as tool errors (`isError`).
    pub fn tool<Args, O, E, F, Fut>(
        mut self,
        name: &'static str,
        description: &'static str,
        f: F,
    ) -> Self
    where
        Args: DeserializeOwned + JsonSchema,
        O: Serialize + JsonSchema,
        E: Serialize,
        F: Fn(Session<S>, Args) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<O, E>> + Send + 'static,
    {
        self.assert_vacant(name);

        let mut spec = json!({
            "name": name,
            "description": description,
            "inputSchema": schema_of::<Args>(),
        });
        let output = schema_of::<O>();
        // MCP requires `outputSchema` (and `structuredContent`) to describe
        // an object; non-object payloads are delivered as text only.
        if output.get("type").is_some_and(|t| t == "object") {
            spec["outputSchema"] = output;
        }

        let run: Run<S> = Box::new(move |session, raw| -> BoxFut<CallOutcome> {
            let args = match serde_json::from_value::<Args>(raw) {
                Ok(args) => args,
                Err(e) => {
                    let msg = format!("invalid arguments: {e}");
                    return Box::pin(std::future::ready(CallOutcome::InvalidArgs(msg)));
                }
            };

            let fut = f(session, args);
            Box::pin(async move {
                match fut.await {
                    Ok(ok) => CallOutcome::Ok(to_value(&ok)),
                    Err(err) => CallOutcome::ToolError(to_value(&err)),
                }
            })
        });

        self.entries.push(Entry { name, spec, run });
        self
    }
}

fn schema_of<T: JsonSchema>() -> Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("schema serialization is infallible")
}

fn to_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("protocol types serialize to JSON")
}
