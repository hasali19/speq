use axum::body::Body;
use axum::handler::Handler;
use axum::{routing, Router};
pub use http::Method;
pub use speq_macros::*;

#[derive(Clone, Copy)]
pub struct RouteRegistrar(pub fn(Router) -> Router);

inventory::collect!(RouteRegistrar);

pub fn router() -> Router {
    inventory::iter::<RouteRegistrar>
        .into_iter()
        .fold(Router::new(), |router, RouteRegistrar(register)| {
            register(router)
        })
}

pub fn register_route<H, T>(router: Router, path: &str, method: Method, route: H) -> Router
where
    H: Handler<T, Body>,
    T: 'static,
{
    router.route(
        path,
        match method {
            Method::GET => routing::get(route),
            Method::POST => routing::post(route),
            Method::PUT => routing::put(route),
            Method::DELETE => routing::delete(route),
            Method::HEAD => routing::head(route),
            Method::OPTIONS => routing::options(route),
            Method::PATCH => routing::patch(route),
            Method::TRACE => routing::trace(route),
            method => panic!("Unsupported method: {}", method),
        },
    )
}
