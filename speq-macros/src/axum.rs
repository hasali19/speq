use std::fmt::Write;

use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{AttributeArgs, Lit, LitInt, LitStr, Meta};
use syn_mid::FnArg;

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

#[derive(StructMeta)]
struct PathArgs {
    #[struct_meta(unnamed)]
    model: Vec<syn::Path>,
}

#[derive(StructMeta)]
struct RequestArgs {
    model: Option<syn::Path>,
}

#[derive(StructMeta)]
struct ResponseArgs {
    status: Option<LitInt>,
    description: Option<LitStr>,
    model: Option<syn::Path>,
}

pub fn route(method: Method, args: TokenStream, mut item: TokenStream) -> TokenStream {
    let mut input: syn_mid::ItemFn = match syn::parse(item.clone()) {
        Ok(input) => input,
        Err(e) => {
            item.extend(TokenStream::from(e.into_compile_error()));
            return item;
        }
    };

    let args = syn::parse_macro_input!(args as AttributeArgs);
    let path = match args.first().unwrap() {
        syn::NestedMeta::Meta(_) => panic!(),
        syn::NestedMeta::Lit(lit) => match lit {
            syn::Lit::Str(path) => path.value(),
            _ => {
                item.extend(TokenStream::from(
                    quote! {compile_error("Invalid path in macro arguments")},
                ));
                return item;
            }
        },
    };

    let name = input.sig.ident.clone();

    let method = match method {
        Method::Get => quote! { axum::http::Method::GET },
        Method::Post => quote! { axum::http::Method::POST },
        Method::Put => quote! { axum::http::Method::PUT },
        Method::Delete => quote! { axum::http::Method::DELETE },
        Method::Head => quote! { axum::http::Method::HEAD },
        Method::Options => quote! { axum::http::Method::OPTIONS },
        Method::Connect => quote! { axum::http::Method::CONNECT },
        Method::Patch => quote! { axum::http::Method::PATCH },
        Method::Trace => quote! { axum::http::Method::TRACE },
    };

    let mut doc = String::new();
    let mut params = vec![];
    let mut query = quote! { None };
    let mut request = quote! { None };
    let mut responses = vec![];

    for attr in &input.attrs {
        if attr.path.is_ident("doc") {
            let meta = attr.parse_meta().unwrap();
            let meta = match meta {
                Meta::NameValue(val) => val,
                _ => unreachable!(),
            };

            let val = match meta.lit {
                Lit::Str(str) => str.value(),
                _ => unreachable!(),
            };

            writeln!(doc, "{}", val).unwrap();
        } else if attr.path.is_ident("path") {
            let args = attr.parse_args::<PathArgs>().unwrap();
            let model = args.model;

            let param_specs = path
                .split('/')
                .filter(|it| it.starts_with(':'))
                .enumerate()
                .map(|(i, name)| {
                    let name = name.trim_start_matches(':');
                    let model = model
                        .get(i)
                        .expect("number of path parameters must match path string");
                    quote! {
                        params.push(speq::ParamSpec {
                            name: #name.into(),
                            type_desc: <#model as speq::reflection::Reflect>::reflect(cx),
                        });
                    }
                });

            params.extend(param_specs);
        } else if attr.path.is_ident("request") {
            let args = attr.parse_args::<RequestArgs>().unwrap();
            let model = args.model;

            request = quote! {
                Some(
                    speq::RequestSpec {
                        type_desc: <#model as speq::reflection::Reflect>::reflect(cx),
                    }
                )
            };
        } else if attr.path.is_ident("response") {
            let args = attr.parse_args::<ResponseArgs>().unwrap();
            let status = args
                .status
                .map(|v| v.base10_parse().unwrap())
                .unwrap_or(200u16);

            let description = match args.description {
                None => quote! { None },
                Some(description) => {
                    quote! { Some(#description.into()) }
                }
            };

            let type_desc = match args.model {
                None => {
                    quote! { None }
                }
                Some(model) => {
                    quote! { Some(<#model as speq::reflection::Reflect>::reflect(cx)) }
                }
            };

            let response_spec = quote! {
                speq::ResponseSpec {
                    status: axum::http::StatusCode::from_u16(#status).unwrap(),
                    description: #description,
                    type_desc: #type_desc,
                }
            };

            responses.push(response_spec);
        }
    }

    input.attrs.retain(|attr| {
        ["path", "request", "response"]
            .iter()
            .all(|ident| !attr.path.is_ident(ident))
    });

    let query_param = input.sig.inputs.iter().find_map(|input| match input {
        FnArg::Receiver(_) => None,
        FnArg::Typed(param) => param
            .attrs
            .iter()
            .find(|it| it.path.is_ident("query"))
            .map(|attr| (param, attr)),
    });

    if let Some((param, _)) = query_param {
        let model = &param.ty;
        query = quote! {
            Some(speq::QuerySpec {
                type_desc: <#model as speq::reflection::Reflect>::reflect(cx)
            })
        };
    }

    for input in input.sig.inputs.iter_mut() {
        if let FnArg::Typed(param) = input {
            param.attrs.retain(|attr| !attr.path.is_ident("query"));
        }
    }

    let doc = if doc.is_empty() {
        quote! { None }
    } else {
        quote! { Some(#doc.into()) }
    };

    TokenStream::from(quote! {
        #input

        const _: () = {
            fn spec(cx: &mut speq::reflection::TypeContext) -> speq::RouteSpec {
                speq::RouteSpec {
                    name: stringify!(#name).into(),
                    path: #path.into(),
                    method: #method,
                    src_file: file!().into(),
                    doc: #doc,
                    params: {
                        let mut params = vec![];
                        #(#params)*
                        params
                    },
                    query: #query,
                    request: #request,
                    responses: vec![#(#responses),*],
                }
            }

            fn register(router: axum::Router<crate::__speq_config::RouterState>) -> axum::Router<crate::__speq_config::RouterState> {
                speq::axum::register_route(router, #path, #method, #name)
            }

            speq::inventory::submit!(speq::RouteSpecFn(spec));
            speq::inventory::submit!(crate::__speq_config::RouteRegistrar(register));
        };
    })
}
