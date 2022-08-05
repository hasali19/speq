use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_reflect(input: TokenStream) -> TokenStream {
    use serde_derive_internals::{ast as serde_ast, attr as serde_attr, Derive};
    let input = syn::parse_macro_input!(input as DeriveInput);
    let cx = serde_derive_internals::Ctxt::new();
    let container = serde_ast::Container::from_ast(&cx, &input, Derive::Serialize).unwrap();
    cx.check().unwrap();

    let ident = container.ident;
    let expr = match container.data {
        serde_ast::Data::Enum(variants) => {
            let tag = match container.attrs.tag() {
                serde_attr::TagType::External => todo!("external tag"),
                serde_attr::TagType::Internal { tag } => tag,
                serde_attr::TagType::Adjacent { .. } => todo!("adjacent tag"),
                serde_attr::TagType::None => todo!("untagged"),
            };

            let variants = variants.into_iter().map(|variant| {
                let name = variant.ident.to_string();
                let serialize_name = variant.attrs.name().serialize_name();
                let kind = match variant.style {
                    serde_ast::Style::Struct => {
                        let fields = variant.fields.into_iter().map(build_field);
                        quote! {
                            EnumVariantKind::Struct(vec![#(#fields),*])
                        }
                    }
                    serde_ast::Style::Tuple => todo!("tuple enum"),
                    serde_ast::Style::Newtype => {
                        let ty = variant.fields[0].ty;
                        quote! {
                            EnumVariantKind::NewType(<#ty as Reflect>::reflect(cx))
                        }
                    }
                    serde_ast::Style::Unit => quote! {
                        EnumVariantKind::Unit
                    },
                };
                quote! {
                    EnumVariant {
                        name: #name.into(),
                        tag_value: #serialize_name.into(),
                        kind: #kind,
                    }
                }
            });

            quote! {
                TypeDecl::Enum(EnumType {
                    name: stringify!(#ident).into(),
                    tag: Some(EnumTag::Internal(#tag.into())),
                    variants: vec![#(#variants),*],
                })
            }
        }
        serde_ast::Data::Struct(style, fields) => match style {
            serde_ast::Style::Struct => {
                let fields = fields.into_iter().map(build_field);
                quote! {
                    TypeDecl::Struct(StructType {
                        name: stringify!(#ident).into(),
                        fields: vec![#(#fields),*],
                    })
                }
            }
            serde_ast::Style::Tuple => todo!(),
            serde_ast::Style::Newtype => todo!(),
            serde_ast::Style::Unit => todo!(),
        },
    };

    TokenStream::from(quote! {
        const _: () = {
            use speq::reflection::*;
            impl Reflect for #ident {
                fn type_id() -> Option<std::borrow::Cow<'static, str>> {
                    Some(concat!(module_path!(), "::", stringify!(#ident)).into())
                }

                fn reflect(cx: &mut TypeContext) -> Type {
                    let id = Self::type_id().unwrap();
                    cx.insert_with(id.clone(), |cx| #expr);
                    Type::Id(id)
                }
            }
        };
    })
}

fn build_field(field: serde_derive_internals::ast::Field) -> proc_macro2::TokenStream {
    let name = field.attrs.name().serialize_name();
    let ty = field.ty;
    let flatten = field.attrs.flatten();
    let required = field.attrs.default().is_none();
    quote! {
        Field {
            name: #name.into(),
            flatten: #flatten,
            required: #required,
            type_desc: <#ty as Reflect>::reflect(cx),
        }
    }
}
