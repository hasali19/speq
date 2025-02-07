use std::fmt::Write;

use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{Expr, ExprLit, FnArg, ItemFn, Lit, LitInt, LitStr};

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
struct ParamArgs {
    skip: bool,
}

#[derive(StructMeta)]
struct ResponseArgs {
    status: Option<LitInt>,
    description: Option<LitStr>,
    model: Option<syn::Path>,
}

pub fn route(method: Method, args: TokenStream, mut item: TokenStream) -> TokenStream {
    let mut input: ItemFn = match syn::parse(item.clone()) {
        Ok(input) => input,
        Err(e) => {
            item.extend(TokenStream::from(e.into_compile_error()));
            return item;
        }
    };

    let path = match syn::parse_macro_input!(args as syn::Lit) {
        syn::Lit::Str(path) => path.value(),
        _ => {
            item.extend(TokenStream::from(
                quote! {compile_error("Invalid path in macro arguments")},
            ));
            return item;
        }
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
    let mut responses = vec![];

    for attr in &input.attrs {
        if attr.path().is_ident("doc") {
            let meta = attr.meta.require_name_value().unwrap();

            let Expr::Lit(ExprLit {
                lit: Lit::Str(str), ..
            }) = &meta.value
            else {
                panic!("invalid path");
            };

            let val = str.value();

            writeln!(doc, "{val}").unwrap();
        } else if attr.path().is_ident("response") {
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

    let inputs = input
        .sig
        .inputs
        .iter()
        .filter_map(|param| match param {
            FnArg::Receiver(_) => None,
            FnArg::Typed(param) => Some(param),
        })
        .filter(|param| {
            let speq_attr = param
                .attrs
                .iter()
                .find(|attr| attr.meta.path().is_ident("speq"));
            let Some(speq_attr) = speq_attr else {
                return true;
            };

            let args: ParamArgs = speq_attr.parse_args().unwrap();

            !args.skip
        })
        .map(|param| {
            let ty = &param.ty;
            quote! {
                <#ty as speq::RouteHandlerInput>::describe(&mut input_cx, &mut spec);
            }
        })
        .collect::<Vec<_>>();

    input.attrs.retain(|attr| {
        ["response"]
            .iter()
            .all(|ident| !attr.path().is_ident(ident))
    });

    for input in input.sig.inputs.iter_mut() {
        if let FnArg::Typed(param) = input {
            param
                .attrs
                .retain(|attr| ["speq"].iter().all(|ident| !attr.path().is_ident(ident)));
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
                let mut spec = speq::RouteSpec {
                    name: stringify!(#name).into(),
                    path: speq::PathSpec {
                        value: #path.into(),
                        params: None,
                    },
                    method: #method,
                    src_file: file!().into(),
                    doc: #doc,
                    query: None,
                    request: None,
                    responses: vec![#(#responses),*],
                };

                let mut input_cx = speq::RouteHandlerInputContext::new(cx);

                #(#inputs)*

                spec
            }

            fn register(router: axum::Router<crate::__speq_config::RouterState>) -> axum::Router<crate::__speq_config::RouterState> {
                speq::axum::register_route(router, #path, #method, #name)
            }

            speq::inventory::submit!(speq::RouteSpecFn(spec));
            speq::inventory::submit!(crate::__speq_config::RouteRegistrar(register));
        };
    })
}
