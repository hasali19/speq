use axum::body::HttpBody;
use axum::handler::Handler;
use axum::{routing, Router};
pub use http::Method;
pub use speq_macros::{
    axum_connect as connect, axum_delete as delete, axum_get as get, axum_head as head,
    axum_options as options, axum_patch as patch, axum_post as post, axum_put as put,
    axum_trace as trace,
};

#[macro_export]
macro_rules! axum_config {
    ($state:ty) => {
        #[doc(hidden)]
        pub(crate) mod __speq_config {
            use super::*;

            pub type RouterState = $state;

            #[derive(Clone, Copy)]
            pub struct RouteRegistrar(
                pub fn(axum::Router<RouterState>) -> axum::Router<RouterState>,
            );

            speq::inventory::collect!(RouteRegistrar);
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! axum_router {
    () => {{
        let router: axum::Router<crate::__speq_config::RouterState> =
            $crate::inventory::iter::<crate::__speq_config::RouteRegistrar>
                .into_iter()
                .fold(
                    axum::Router::new(),
                    |router, crate::__speq_config::RouteRegistrar(register)| register(router),
                );
        router
    }};
}

#[doc(hidden)]
pub fn register_route<H, T, S, B>(
    router: Router<S, B>,
    path: &str,
    method: Method,
    route: H,
) -> Router<S, B>
where
    H: Handler<T, S, B>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    B: HttpBody + Send + 'static,
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
            method => panic!("Unsupported method: {method}"),
        },
    )
}
