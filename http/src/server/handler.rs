use eva::fut::Fut;

use serde::Deserialize;

use std::{marker::PhantomData, sync::Arc};

use axum::{
    Router as AxumRouter,
    extract::Request as AxumRequest,
    response::Response as AxumResponse,
    routing::{MethodFilter, method_routing},
};

use viendesu_core::{
    errors as core_errors,
    service::{Session, SessionOf, authz::Authentication as _},
};

use crate::server::{
    State, Types,
    context::{Context, load_args},
    request::{ServerRequest, extract},
    response,
};

struct Inner<R: ServerRequest, Cx> {
    make_context: Cx,
    method: MethodFilter,
    _phantom: PhantomData<R>,
}

pub struct RouterScope<T: Types> {
    router: AxumRouter,
    state: State<T>,
}

impl<T: Types> RouterScope<T> {
    pub fn root(state: State<T>) -> Self {
        Self {
            router: AxumRouter::new(),
            state,
        }
    }

    pub fn include(self, f: impl FnOnce(RouterScope<T>) -> RouterScope<T>) -> RouterScope<T> {
        f(self)
    }

    pub fn route<R: ServerRequest, M, Cx>(
        self,
        path: &str,
        handler: FinishedHandler<R, M, T, Cx>,
    ) -> RouterScope<T>
    where
        M: MakeRequest<T, R>,
        Cx: MakeContext<R>,
    {
        let Self { mut router, state } = self;
        router = router.route(
            path,
            method_routing::on(
                handler.inner.method,
                handler.into_axum_handler(state.clone()),
            ),
        );

        Self { router, state }
    }

    pub fn nest(
        self,
        path: &str,
        f: impl FnOnce(RouterScope<T>) -> RouterScope<T>,
    ) -> RouterScope<T> {
        let nested = f(RouterScope {
            router: AxumRouter::new(),
            state: self.state.clone(),
        });

        let Self { mut router, state } = self;
        router = router.nest(path, nested.router);

        Self { router, state }
    }

    pub fn map_axum(self, f: impl FnOnce(AxumRouter) -> AxumRouter) -> Self {
        let Self { router, state } = self;
        Self {
            router: f(router),
            state,
        }
    }

    pub fn into_axum(self) -> AxumRouter {
        self.router
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
                    method: _,
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
            method: MethodFilter::GET,
            _phantom: PhantomData,
        })
    }

    pub fn post(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            method: MethodFilter::POST,
            _phantom: PhantomData,
        })
    }

    pub fn patch(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            method: MethodFilter::PATCH,
            _phantom: PhantomData,
        })
    }

    pub fn delete(make_context: Cx) -> Handler<R, Cx> {
        Self(Inner {
            make_context,
            method: MethodFilter::DELETE,
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
