#[cfg(feature = "axum")]
pub mod axum;
pub mod reflection;

use std::borrow::Cow;
use std::collections::HashMap;

pub use http::{Method, StatusCode};
pub use inventory;
pub use speq_macros::*;

use reflection::{Type, TypeContext, TypeDecl};

#[cfg(all(feature = "axum_query", not(feature = "axum")))]
compile_error!("feature 'axum_query' requires also enabling 'axum'");

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

inventory::collect!(RouteSpecFn);

pub fn spec() -> ApiSpec {
    let mut tcx = TypeContext::new();

    let mut routes = vec![];
    for RouteSpecFn(f) in inventory::iter::<RouteSpecFn> {
        routes.push(f(&mut tcx));
    }

    ApiSpec {
        routes,
        types: tcx.into_types(),
    }
}
