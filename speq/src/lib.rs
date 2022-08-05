pub mod reflection;

pub use axum::http::{Method, StatusCode};
pub use speq_macros::*;

use reflection::{Type, TypeContext};

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

pub trait Route: Send + Sync {
    fn name(&self) -> &'static str;
    fn path(&self) -> &'static str;
    fn method(&self) -> Method;

    fn src_file(&self) -> &'static str;

    fn doc(&self) -> Option<&'static str> {
        None
    }

    fn params(&self, cx: &mut TypeContext) -> Vec<ParamSpec>;
    fn query(&self, cx: &mut TypeContext) -> Option<QuerySpec>;
    fn request(&self, cx: &mut TypeContext) -> Option<RequestSpec>;
    fn responses(&self, cx: &mut TypeContext) -> Vec<ResponseSpec>;

    fn register(&self, router: axum::Router) -> axum::Router;
}

pub mod inventory {
    pub use inventory::submit;
}

#[macro_export]
macro_rules! submit {
    ($e:expr) => {
        speq::inventory::submit! {
            $e as &'static dyn speq::Route
        }
    };
}

::inventory::collect!(&'static dyn Route);

pub fn routes() -> impl Iterator<Item = &'static dyn Route> {
    ::inventory::iter::<&'static dyn Route>.into_iter().copied()
}
