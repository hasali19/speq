pub mod reflection;

use std::borrow::Cow;
use std::collections::HashMap;

pub use axum::http::{Method, StatusCode};
pub use inventory;
pub use speq_macros::*;

use reflection::{Type, TypeContext, TypeDecl};

#[derive(Clone, Debug)]
pub struct ParamSpec {
    pub name: String,
    pub type_desc: Type,
}

#[derive(Clone, Debug)]
pub struct QuerySpec {
    pub type_desc: Type,
}

#[derive(Clone, Debug)]
pub struct RequestSpec {
    pub type_desc: Type,
}

#[derive(Clone, Debug)]
pub struct ResponseSpec {
    pub status: StatusCode,
    pub description: Option<String>,
    pub type_desc: Option<Type>,
}

#[derive(Clone, Debug)]
pub struct RouteSpec {
    pub name: Cow<'static, str>,
    pub path: Cow<'static, str>,
    pub method: Method,
    pub src_file: Cow<'static, str>,
    pub doc: Option<Cow<'static, str>>,
    pub params: Vec<ParamSpec>,
    pub query: Option<QuerySpec>,
    pub request: Option<RequestSpec>,
    pub responses: Vec<ResponseSpec>,
}

#[derive(Clone, Debug)]
pub struct ApiSpec {
    pub routes: Vec<RouteSpec>,
    pub types: HashMap<Cow<'static, str>, TypeDecl>,
}

#[derive(Clone, Copy)]
pub struct RouteSpecFn(pub fn(&mut TypeContext) -> RouteSpec);

impl RouteSpecFn {
    pub fn build(&self, cx: &mut TypeContext) -> RouteSpec {
        self.0(cx)
    }
}

#[derive(Clone, Copy)]
pub struct RouteRegistrar(pub fn(axum::Router) -> axum::Router);

::inventory::collect!(RouteSpecFn);
::inventory::collect!(RouteRegistrar);

pub fn spec() -> ApiSpec {
    let mut tcx = TypeContext::new();

    let mut routes = vec![];
    for RouteSpecFn(f) in ::inventory::iter::<RouteSpecFn> {
        routes.push(f(&mut tcx));
    }

    ApiSpec {
        routes,
        types: tcx.into_types(),
    }
}

pub fn router() -> axum::Router {
    ::inventory::iter::<RouteRegistrar>
        .into_iter()
        .fold(axum::Router::new(), |router, RouteRegistrar(register)| {
            register(router)
        })
}

pub fn register_route<H, T>(
    router: axum::Router,
    path: &str,
    method: Method,
    route: H,
) -> axum::Router
where
    H: axum::handler::Handler<T, axum::body::Body>,
    T: 'static,
{
    router.route(
        path,
        match method {
            axum::http::Method::GET => axum::routing::get(route),
            axum::http::Method::POST => axum::routing::post(route),
            axum::http::Method::PUT => axum::routing::put(route),
            axum::http::Method::DELETE => axum::routing::delete(route),
            axum::http::Method::HEAD => axum::routing::head(route),
            axum::http::Method::OPTIONS => axum::routing::options(route),
            axum::http::Method::PATCH => axum::routing::patch(route),
            axum::http::Method::TRACE => axum::routing::trace(route),
            method => panic!("Unsupported method: {}", method),
        },
    )
}
