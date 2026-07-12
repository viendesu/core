use eva::fut::Fut;

use serde::Deserialize;

use std::{marker::PhantomData, sync::Arc};

use axum::{
    Router as AxumRouter,
    extract::Request as AxumRequest,
    response::Response as AxumResponse,
    routing::{MethodFilter, method_routing},
};

use viendesu_core::service::{Session, SessionOf, authz::Authentication as _};
use viendesu_protocol::errors as core_errors;

use crate::server::{
    State, Types,
    context::{Context, load_args},
    openapi,
    request::ServerRequest,
    response,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Get,
    Post,
    Patch,
    Delete,
}

impl Verb {
    fn filter(self) -> MethodFilter {
        match self {
            Self::Get => MethodFilter::GET,
            Self::Post => MethodFilter::POST,
            Self::Patch => MethodFilter::PATCH,
            Self::Delete => MethodFilter::DELETE,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }
}

struct Inner<R: ServerRequest, Cx> {
    make_context: Cx,
    verb: Verb,
    _phantom: PhantomData<R>,
}

pub struct RouterScope<T: Types> {
    router: AxumRouter,
    state: State<T>,
    prefix: String,
    api: openapi::Collector,
}

impl<T: Types> RouterScope<T> {
    pub fn root(state: State<T>) -> Self {
        Self {
            router: AxumRouter::new(),
            state,
            prefix: String::new(),
            api: openapi::Collector::new(),
        }
    }

    pub fn include(self, f: impl FnOnce(RouterScope<T>) -> RouterScope<T>) -> RouterScope<T> {
        f(self)
    }

    pub fn route<R: ServerRequest, M, Cx>(
        mut self,
        path: &str,
        handler: FinishedHandler<R, M, T, Cx>,
    ) -> RouterScope<T>
    where
        M: MakeRequest<T, R>,
        Cx: MakeContext<R>,
    {
        let verb = handler.inner.verb;
        self.api
            .operation::<R>(verb, openapi::join_path(&self.prefix, path));

        let Self {
            mut router,
            state,
            prefix,
            api,
        } = self;
        router = router.route(
            path,
            method_routing::on(verb.filter(), handler.into_axum_handler(state.clone())),
        );

        Self {
            router,
            state,
            prefix,
            api,
        }
    }

    pub fn nest(
        self,
        path: &str,
        f: impl FnOnce(RouterScope<T>) -> RouterScope<T>,
    ) -> RouterScope<T> {
        let Self {
            router,
            state,
            prefix,
            api,
        } = self;

        let nested = f(RouterScope {
            router: AxumRouter::new(),
            state: state.clone(),
            prefix: format!("{prefix}{path}"),
            api,
        });

        Self {
            router: router.nest(path, nested.router),
            state,
            prefix,
            api: nested.api,
        }
    }

    pub fn map_axum(self, f: impl FnOnce(AxumRouter) -> AxumRouter) -> Self {
        Self {
            router: f(self.router),
            ..self
        }
    }

    pub fn finish(self) -> (AxumRouter, openapi::Collector) {
        (self.router, self.api)
    }
}

pub struct FinishedHandler<R: ServerRequest, M, T: Types, Cx> {
    inner: Inner<R, Cx>,
    make_request: M,
    _phantom: PhantomData<T>,
}

impl<R: ServerRequest, M, T: Types, Cx: MakeContext<R>> FinishedHandler<R, M, T, Cx> {
    fn into_axum_handler(self, state: State<T>) -> impl AxumHandler
    where
        M: MakeRequest<T, R>,
    {
        let Self {
            inner,
            make_request,
            _phantom: _,
        } = self;

        let captures = Arc::new((make_request, inner, state));

        move |req: AxumRequest| {
            let captures = captures.clone();
            async move {
                let (make_request, inner, state) = Arc::as_ref(&captures);
                let Inner {
                    make_context,
                    verb: _,
                    _phantom: _,
                } = inner;

                let (parts, body) = req.into_parts();
                let context = make_context(AxumRequest::from_parts(parts, body)).await?;
                let resp_format = context.response_format;
                let mut session = state
                    .make_session()
                    .await
                    .map_err(|e| response::err(resp_format, e))?;

                if let Some(token) = context.token {
                    session
                        .authz()
                        .authenticate(token)
                        .await
                        .map_err(|e| response::err(resp_format, e))?;
                }

                match make_request(session, context).await {
                    Ok(r) => Ok(response::ok(resp_format, r)),
                    Err(e) => Err(response::err(resp_format, e)),
                }
            }
        }
    }
}

pub fn get<R, T, M>(make_request: M) -> FinishedHandler<R, M, T, impl MakeContext<R>>
where
    R: ServerRequest + for<'de> Deserialize<'de>,
    T: Types,
    M: MakeRequest<T, R>,
{
    Handler::get(load_args::<R>).exec(make_request)
}

pub fn post<R, T, M>(make_request: M) -> FinishedHandler<R, M, T, impl MakeContext<R>>
where
    R: ServerRequest + for<'de> Deserialize<'de>,
    T: Types,
    M: MakeRequest<T, R>,
{
    Handler::post(load_args::<R>).exec(make_request)
}

pub fn patch<R, T, M>(make_request: M) -> FinishedHandler<R, M, T, impl MakeContext<R>>
where
    R: ServerRequest + for<'de> Deserialize<'de>,
    T: Types,
    M: MakeRequest<T, R>,
{
    Handler::patch(load_args::<R>).exec(make_request)
}

pub fn delete<R, T, M>(make_request: M) -> FinishedHandler<R, M, T, impl MakeContext<R>>
where
    R: ServerRequest + for<'de> Deserialize<'de>,
    T: Types,
    M: MakeRequest<T, R>,
{
    Handler::delete(load_args::<R>).exec(make_request)
}

pub struct Handler<R: ServerRequest, Cx>(Inner<R, Cx>);

impl<R: ServerRequest, Cx: MakeContext<R>> Handler<R, Cx> {
    pub fn get(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            verb: Verb::Get,
            _phantom: PhantomData,
        })
    }

    pub fn post(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            verb: Verb::Post,
            _phantom: PhantomData,
        })
    }

    pub fn patch(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            verb: Verb::Patch,
            _phantom: PhantomData,
        })
    }

    pub fn delete(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            verb: Verb::Delete,
            _phantom: PhantomData,
        })
    }

    pub fn exec<T, M>(self, make_request: M) -> FinishedHandler<R, M, T, Cx>
    where
        T: Types,
        M: MakeRequest<T, R>,
    {
        let Self(inner) = self;
        FinishedHandler {
            inner,
            make_request,
            _phantom: PhantomData,
        }
    }
}

// == Complex trait magic ==

pub trait AxumHandler: Send + Sync + 'static + Clone + Fn(AxumRequest) -> Self::AFut {
    #[doc(hidden)]
    type AFut: Fut<Output = Result<AxumResponse, AxumResponse>>;
}

impl<H, F> AxumHandler for H
where
    H: Clone + Send + Sync + 'static + Fn(AxumRequest) -> F,
    F: Fut<Output = Result<AxumResponse, AxumResponse>>,
{
    type AFut = F;
}

pub trait Captures<T: ?Sized> {}
impl<T: ?Sized, U: ?Sized> Captures<T> for U {}

pub trait MakeContext<R: ServerRequest>:
    Send + Sync + 'static + Fn(AxumRequest) -> Self::MkFut
{
    type MkFut: Fut<Output = Result<Context<R>, AxumResponse>>;
}

impl<R: ServerRequest, H, F> MakeContext<R> for H
where
    H: Send + Sync + 'static + Fn(AxumRequest) -> F,
    F: Fut<Output = Result<Context<R>, AxumResponse>>,
{
    type MkFut = F;
}

pub trait MakeRequest<T: Types, R: ServerRequest>:
    Send + Sync + 'static + Fn(Session<SessionOf<T::Service>>, Context<R>) -> Self::HFut
{
    type HFut: Fut<Output = core_errors::Result<R::Response, R::Error>>
        + Captures<SessionOf<T::Service>>;
}

impl<T, R, F, H> MakeRequest<T, R> for H
where
    H: Send + Sync + 'static + Fn(Session<SessionOf<T::Service>>, Context<R>) -> F,
    F: Fut<Output = core_errors::Result<R::Response, R::Error>> + Captures<SessionOf<T::Service>>,
    T: Types,
    R: ServerRequest,
{
    type HFut = F;
}
