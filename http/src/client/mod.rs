use std::sync::Arc;

use serde::{Deserialize, Serialize};

use eva::{
    error::ShitHappens,
    str::{CompactString, format_compact as c},
};

use http::Method;
use viendesu_core::{
    errors::{self, Aux},
    requests::Response,
    service::{
        CallStep, Session, SessionMaker,
        authors::Authors,
        authz::Authentication,
        boards::Boards,
        games::Games,
        marks::{Badges, Genres, Tags},
        messages::Messages,
        tabs::Tabs,
        threads::Threads,
        uploads::Uploads,
        users::Users,
    },
    types::session,
};

use crate::{format::Format, requests::Request};

mod boards;
mod messages;
mod threads;

mod auth;

mod authors;
mod games;
mod users;

mod marks;
mod tabs;

mod uploads;

struct DoRequest<'c, P> {
    client: &'c mut HttpClient,
    method: Method,
    map_payload: P,
}

impl<'c, P, C, H> CallStep<C> for DoRequest<'c, P>
where
    C: Send + Sync,
    P: Send + Sync + FnMut(C) -> (CompactString, H),
    H: Request + Serialize,
{
    type Ok = H::Response;
    type Err = H::Error;

    async fn call(&mut self, req: C) -> Response<Self::Ok, Self::Err> {
        let (path, request) = (self.map_payload)(req);
        self.client
            .do_request(self.method.clone(), &path, request)
            .await
    }
}

fn todo<C, O>() -> impl Send + Sync + FnMut(C) -> (CompactString, O) {
    |_| todo!()
}

pub struct HttpClient {
    options: Arc<ClientOptions>,
    client: reqwest::Client,
    session: Option<session::Token>,
}

impl HttpClient {
    fn do_call<'this, P>(&'this mut self, method: Method, map_payload: P) -> DoRequest<'this, P> {
        DoRequest {
            client: self,
            method,
            map_payload,
        }
    }

    fn endpoint(&self, path: &str) -> String {
        let mut endpoint = self.options.endpoint.clone();
        if endpoint.ends_with('/') {
            endpoint.push_str(path.strip_prefix('/').unwrap_or(path));
        } else {
            endpoint.push_str(path);
        }

        endpoint
    }

    fn load_response<O, E>(&self, data: &[u8]) -> Response<O, E>
    where
        O: for<'de> Deserialize<'de>,
        E: for<'de> Deserialize<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum GenericResponse<O, E> {
            Success { ok: O },
            Error { error: errors::Generic<E> },
        }

        let resp: GenericResponse<O, E> =
            self.options.format.load(data).map_err(|e| {
                Aux::Deserialization(format!("failed to deserialize response: {e:#}"))
            })?;

        match resp {
            GenericResponse::Error { error } => Err(error),
            GenericResponse::Success { ok } => Ok(ok),
        }
    }

    async fn do_request<R>(
        &self,
        method: Method,
        path: &str,
        request: R,
    ) -> Response<R::Response, R::Error>
    where
        R: Request + Serialize,
    {
        let mut req = self.client.request(method.clone(), self.endpoint(path));
        if method == Method::GET {
            req = req.query(&request);
        } else {
            let mut dst = Vec::with_capacity(128);
            self.options
                .format
                .dump(Default::default(), &request, &mut dst);
            req = req
                .body(dst)
                .header("content-type", self.options.format.mime_type());
        }

        if let Some(sess) = self.session {
            req = req.bearer_auth(sess);
        }

        let response = self
            .client
            .execute(req.build().expect("shit happens"))
            .await
            .map_err(|e| Aux::InternalError(format!("failed to make request: {e:#}")))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Aux::InternalError(format!("failed to read bytes: {e:#}")))?;
        self.load_response(&bytes)
    }
}

pub struct ClientOptions {
    pub format: Format,
    pub endpoint: String,
}

#[derive(Clone)]
pub struct HttpService {
    options: Arc<ClientOptions>,
    inner: reqwest::Client,
}

impl SessionMaker for HttpService {
    type Session = HttpClient;

    async fn make_session(&self) -> errors::AuxResult<Session<Self::Session>> {
        Ok(Session::new(HttpClient {
            options: Arc::clone(&self.options),
            client: self.inner.clone(),
            session: None,
        }))
    }
}

impl HttpService {
    pub fn new(options: ClientOptions) -> Self {
        let inner = reqwest::Client::builder()
            .user_agent("Makise/HTTPP")
            .default_headers({
                use reqwest::header;

                let mut headers = header::HeaderMap::new();
                headers.insert(
                    "Accept",
                    header::HeaderValue::from_static(options.format.mime_type()),
                );

                headers
            })
            .build()
            .shit_happens();

        Self {
            inner,
            options: Arc::new(options),
        }
    }
}
