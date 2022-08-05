use proc_macro::TokenStream;

#[cfg(feature = "axum")]
mod axum;
mod derive;

#[proc_macro_derive(Reflect, attributes(serde))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    derive::derive_reflect(input)
}

macro_rules! axum_route_macro {
    ($name:ident, $method:ident) => {
        #[cfg(feature = "axum")]
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, item: TokenStream) -> TokenStream {
            axum::route(axum::Method::$method, args, item)
        }
    };
}

axum_route_macro!(axum_get, Get);
axum_route_macro!(axum_post, Post);
axum_route_macro!(axum_put, Put);
axum_route_macro!(axum_delete, Delete);
axum_route_macro!(axum_head, Head);
axum_route_macro!(axum_options, Options);
axum_route_macro!(axum_connect, Connect);
axum_route_macro!(axum_patch, Patch);
axum_route_macro!(axum_trace, Trace);
