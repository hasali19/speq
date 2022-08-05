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

axum_route_macro!(get, Get);
axum_route_macro!(post, Post);
axum_route_macro!(put, Put);
axum_route_macro!(delete, Delete);
axum_route_macro!(head, Head);
axum_route_macro!(options, Options);
axum_route_macro!(connect, Connect);
axum_route_macro!(patch, Patch);
axum_route_macro!(trace, Trace);
