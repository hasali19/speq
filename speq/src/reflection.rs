use std::collections::HashMap;

use crate::SpeqStr;

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct TypeContext {
    types: HashMap<SpeqStr, TypeDecl>,
}

impl TypeContext {
    pub fn new() -> TypeContext {
        TypeContext {
            types: HashMap::new(),
        }
    }

    pub fn insert_with(&mut self, id: SpeqStr, f: impl FnOnce(&mut Self) -> TypeDecl) {
        #[allow(clippy::map_entry)]
        if !self.types.contains_key(&id) {
            let decl = f(self);
            self.types.insert(id, decl);
        }
    }

    pub fn into_types(self) -> HashMap<SpeqStr, TypeDecl> {
        self.types
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Reflect {
    fn type_id() -> Option<SpeqStr>;
    fn reflect(cx: &mut TypeContext) -> Type;
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Option(Box<Type>),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Map(Box<Type>),
    Id(SpeqStr),
}

impl Type {
    pub fn as_id(&self) -> Option<&str> {
        if let Type::Id(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeDecl {
    Struct(StructType),
    Enum(EnumType),
}

impl TypeDecl {
    pub fn as_struct(&self) -> Option<&StructType> {
        if let Self::Struct(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IntWidth {
    W8 = 8,
    W16 = 16,
    W32 = 32,
    W64 = 64,
    W128 = 128,
}

impl IntWidth {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum FloatWidth {
    F32 = 32,
    F64 = 64,
}

impl FloatWidth {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug)]
pub enum PrimitiveType {
    Bool,
    Int(IntWidth),
    UInt(IntWidth),
    Float(FloatWidth),
    String,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: SpeqStr,
    pub flatten: bool,
    pub required: bool,
    pub type_desc: Type,
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub name: SpeqStr,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug)]
pub enum EnumTag {
    External,
    Internal(SpeqStr),
    Adjacent { tag: SpeqStr, content: SpeqStr },
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: SpeqStr,
    pub tag_value: SpeqStr,
    pub kind: EnumVariantKind,
}

#[derive(Clone, Debug)]
pub enum EnumVariantKind {
    Unit,
    NewType(Type),
    Struct(Vec<Field>),
}

#[derive(Clone, Debug)]
pub struct EnumType {
    pub name: SpeqStr,
    pub tag: Option<EnumTag>,
    pub variants: Vec<EnumVariant>,
}

mod impls {
    use super::*;

    macro_rules! impl_for_primitive {
        ($t:ty, $e:expr) => {
            impl Reflect for $t {
                fn type_id() -> Option<SpeqStr> {
                    None
                }

                fn reflect(_: &mut TypeContext) -> Type {
                    Type::Primitive($e)
                }
            }
        };
    }

    impl_for_primitive!(i8, PrimitiveType::Int(IntWidth::W8));
    impl_for_primitive!(i16, PrimitiveType::Int(IntWidth::W16));
    impl_for_primitive!(i32, PrimitiveType::Int(IntWidth::W32));
    impl_for_primitive!(i64, PrimitiveType::Int(IntWidth::W64));
    impl_for_primitive!(i128, PrimitiveType::Int(IntWidth::W128));

    impl_for_primitive!(u8, PrimitiveType::UInt(IntWidth::W8));
    impl_for_primitive!(u16, PrimitiveType::UInt(IntWidth::W16));
    impl_for_primitive!(u32, PrimitiveType::UInt(IntWidth::W32));
    impl_for_primitive!(u64, PrimitiveType::UInt(IntWidth::W64));
    impl_for_primitive!(u128, PrimitiveType::UInt(IntWidth::W128));

    impl_for_primitive!(f32, PrimitiveType::Float(FloatWidth::F32));
    impl_for_primitive!(f64, PrimitiveType::Float(FloatWidth::F64));

    impl_for_primitive!(bool, PrimitiveType::Bool);
    impl_for_primitive!(String, PrimitiveType::String);

    macro_rules! impl_for_tuple {
        ($($t:ident)*) => {
            #[allow(unused)]
            impl<$($t: Reflect,)*> Reflect for ($($t,)*) {
                fn type_id() -> Option<SpeqStr> {
                    None
                }

                fn reflect(cx: &mut TypeContext) -> Type {
                    Type::Tuple(vec![$($t::reflect(cx),)*])
                }
            }
        };
    }

    impl_for_tuple!();
    impl_for_tuple!(A);
    impl_for_tuple!(A B);
    impl_for_tuple!(A B C D);
    impl_for_tuple!(A B C D E);
    impl_for_tuple!(A B C D E F);
    impl_for_tuple!(A B C D E F G);
    impl_for_tuple!(A B C D E F G H);
    impl_for_tuple!(A B C D E F G H I);
    impl_for_tuple!(A B C D E F G H I J);
    impl_for_tuple!(A B C D E F G H I J K);
    impl_for_tuple!(A B C D E F G H I J K L);

    impl<T: Reflect> Reflect for Option<T> {
        fn type_id() -> Option<SpeqStr> {
            None
        }

        fn reflect(cx: &mut TypeContext) -> Type {
            Type::Option(Box::new(T::reflect(cx)))
        }
    }

    impl<T: Reflect> Reflect for Vec<T> {
        fn type_id() -> Option<SpeqStr> {
            None
        }

        fn reflect(cx: &mut TypeContext) -> Type {
            Type::Array(Box::new(T::reflect(cx)))
        }
    }

    #[allow(unused)]
    macro_rules! forward_impl {
        ($type:ty) => {
            impl<T: Reflect> Reflect for $type {
                fn type_id() -> Option<SpeqStr> {
                    T::type_id()
                }

                fn reflect(cx: &mut TypeContext) -> Type {
                    T::reflect(cx)
                }
            }
        };
    }

    #[cfg(feature = "camino")]
    impl Reflect for camino::Utf8Path {
        fn type_id() -> Option<SpeqStr> {
            Some(SpeqStr::Borrowed("camino::Utf8Path"))
        }

        fn reflect(cx: &mut TypeContext) -> Type {
            String::reflect(cx)
        }
    }

    #[cfg(feature = "camino")]
    impl Reflect for camino::Utf8PathBuf {
        fn type_id() -> Option<SpeqStr> {
            Some(SpeqStr::Borrowed("camino::Utf8PathBuf"))
        }

        fn reflect(cx: &mut TypeContext) -> Type {
            String::reflect(cx)
        }
    }
}
