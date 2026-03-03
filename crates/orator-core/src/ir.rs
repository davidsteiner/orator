use std::collections::BTreeMap;

/// A named type from `components/schemas` in the OpenAPI spec.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub description: Option<String>,
    pub kind: TypeDefKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    Struct(StructDef),
    StringEnum(StringEnumDef),
    Alias(TypeRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub bases: Vec<TypeRef>,
    pub fields: Vec<Field>,
    pub variants: Vec<Variant>,
    pub discriminator: Option<DiscriminatorDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringEnumDef {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscriminatorDef {
    pub property: String,
    pub mapping: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_ref: TypeRef,
    pub required: bool,
    pub description: Option<String>,
}

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    String,
    Bool,
    I32,
    I64,
    F32,
    F64,
}
