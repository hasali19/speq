#[cfg(feature = "axum")]
pub mod axum;
pub mod reflection;

use std::borrow::Cow;
use std::collections::HashMap;

pub use http::{Method, StatusCode};
pub use inventory;
pub use speq_macros::Reflect;

use reflection::{Type, TypeContext, TypeDecl};

pub type SpeqStr = Cow<'static, str>;

#[cfg(all(feature = "axum_query", not(feature = "axum")))]
compile_error!("feature 'axum_query' requires also enabling 'axum'");

#[derive(Clone, Debug)]
pub struct PathSpec {
    pub value: SpeqStr,
    pub params: Vec<Type>,
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
    pub description: Option<SpeqStr>,
    pub type_desc: Option<Type>,
}

#[derive(Clone, Debug)]
pub struct RouteSpec {
    pub name: SpeqStr,
    pub path: PathSpec,
    pub method: Method,
    pub src_file: SpeqStr,
    pub doc: Option<SpeqStr>,
    pub query: Option<QuerySpec>,
    pub request: Option<RequestSpec>,
    pub responses: Vec<ResponseSpec>,
}

#[derive(Clone, Debug)]
pub struct ApiSpec {
    pub routes: Vec<RouteSpec>,
    pub types: HashMap<SpeqStr, TypeDecl>,
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
