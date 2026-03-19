use std::collections::BTreeMap;

/// A named type from `components/schemas` in the OpenAPI spec.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub description: Option<String>,
    pub kind: TypeDefKind,
}

/// The shape of a named type.
///
/// - [`Struct`](TypeDefKind::Struct): has properties and/or `allOf` bases.
/// - [`Enum`](TypeDefKind::Enum): pure `oneOf`/`anyOf` with no additional fields.
/// - [`StringEnum`](TypeDefKind::StringEnum): `type: string` with `enum` values.
/// - [`Alias`](TypeDefKind::Alias): wraps another type (primitives, arrays, bare `$ref`).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    Struct(StructDef),
    Enum(EnumDef),
    StringEnum(StringEnumDef),
    Alias(TypeRef),
}

/// The representation of a struct in Rust.
#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub bases: Vec<TypeRef>,
    pub fields: Vec<Field>,
    pub deny_unknown_fields: bool,
}

/// The representation of an enum in Rust.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub variants: Vec<Variant>,
    pub discriminator: Option<DiscriminatorDef>,
}

/// A `type: string` schema with `enum` values, generating a Rust enum with unit variants.
#[derive(Debug, Clone, PartialEq)]
pub struct StringEnumDef {
    pub values: Vec<String>,
}

/// The discriminator object in OpenAPI for `oneOf` and `anyOf` variants.
///
/// `property` is the field name to discriminate on.
/// `mapping` is the mapping from discriminator tags to schema names.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscriminatorDef {
    pub property: String,
    pub mapping: BTreeMap<String, String>,
}

/// A property on a struct.
///
/// Non-required fields are wrapped in `Option<T>` during codegen.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_ref: TypeRef,
    pub required: bool,
    pub description: Option<String>,
}

/// A branch of either `oneOf` or `anyOf`.
///
/// It gets translated into an enum variant in Rust.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub type_ref: TypeRef,
    pub mapping_value: Option<String>,
}

/// A reference to a type, used for inline schemas (properties, array items, etc.)
/// that don't have their own named [`TypeDef`].
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Named(String),
    Primitive(PrimitiveType),
    Array(Box<TypeRef>),
    Option(Box<TypeRef>),
    Map(Box<TypeRef>),
    Any,
}

/// A Rust primitive type, derived from the `type` and `format` fields in the OpenAPI spec.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    String,
    Bool,
    I32,
    I64,
    F32,
    F64,
    Bytes,
    Date,
    DateTime,
    Uuid,
}
