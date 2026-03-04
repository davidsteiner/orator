use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use crate::ir::{
    DiscriminatorDef, EnumDef, PrimitiveType, StringEnumDef, StructDef, TypeDef, TypeDefKind,
    TypeRef, Variant,
};

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

/// Generate Rust source code for a list of type definitions.
pub fn generate_types(types: &[TypeDef]) -> String {
    let items: Vec<TokenStream> = types.iter().map(generate_typedef).collect();
    let file_tokens = quote! { #(#items)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    prettyplease::unparse(&syntax_tree)
}

fn generate_typedef(typedef: &TypeDef) -> TokenStream {
    let doc = generate_doc_comment(&typedef.description);
    match &typedef.kind {
        TypeDefKind::Struct(s) => generate_struct(&typedef.name, s, doc),
        TypeDefKind::Enum(e) => generate_enum(&typedef.name, e, doc),
        TypeDefKind::StringEnum(se) => generate_string_enum(&typedef.name, se, doc),
        TypeDefKind::Alias(type_ref) => generate_alias(&typedef.name, type_ref, doc),
    }
}

fn generate_struct(name: &str, def: &StructDef, doc: TokenStream) -> TokenStream {
    let struct_ident = to_pascal_ident(name);

    let base_fields: Vec<TokenStream> = def
        .bases
        .iter()
        .map(|base| {
            let base_type = type_ref_to_tokens(base);
            let field_ident = match base {
                TypeRef::Named(n) => to_snake_ident(n),
                _ => unreachable!("bases are always Named refs"),
            };
            quote! {
                #[serde(flatten)]
                pub #field_ident: #base_type,
            }
        })
        .collect();

    let regular_fields: Vec<TokenStream> = def
        .fields
        .iter()
        .map(|field| {
            let field_ident = to_snake_ident(&field.name);
            let field_doc = generate_doc_comment(&field.description);

            let snake_case_name = field.name.to_snake_case();
            let rename_attr = if snake_case_name != field.name {
                let original = &field.name;
                quote! { #[serde(rename = #original)] }
            } else {
                quote! {}
            };

            let (field_type, skip_attr) = if field.required {
                (type_ref_to_tokens(&field.type_ref), quote! {})
            } else {
                let wrapped = match &field.type_ref {
                    TypeRef::Option(_) => type_ref_to_tokens(&field.type_ref),
                    other => {
                        let inner = type_ref_to_tokens(other);
                        quote! { Option<#inner> }
                    }
                };
                (
                    wrapped,
                    quote! { #[serde(skip_serializing_if = "Option::is_none")] },
                )
            };

            quote! {
                #field_doc
                #rename_attr
                #skip_attr
                pub #field_ident: #field_type,
            }
        })
        .collect();

    quote! {
        #doc
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct #struct_ident {
            #(#base_fields)*
            #(#regular_fields)*
        }
    }
}

fn generate_enum(name: &str, def: &EnumDef, doc: TokenStream) -> TokenStream {
    let enum_ident = to_pascal_ident(name);

    let serde_attr = if let Some(disc) = &def.discriminator {
        let tag = &disc.property;
        quote! { #[serde(tag = #tag)] }
    } else {
        quote! { #[serde(untagged)] }
    };

    let variants: Vec<TokenStream> = def
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = variant_name_for_type_ref(&variant.type_ref);
            let variant_type = type_ref_to_tokens(&variant.type_ref);

            let rename_attr = variant_rename_attr(variant, &def.discriminator, &variant_ident);

            quote! {
                #rename_attr
                #variant_ident(#variant_type),
            }
        })
        .collect();

    quote! {
        #doc
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        #serde_attr
        pub enum #enum_ident {
            #(#variants)*
        }
    }
}

fn variant_rename_attr(
    variant: &Variant,
    discriminator: &Option<DiscriminatorDef>,
    variant_ident: &Ident,
) -> TokenStream {
    // if there's an explicit mapping value, use it
    if let Some(val) = &variant.mapping_value {
        if val != &variant_ident.to_string() {
            return quote! { #[serde(rename = #val)] };
        }
    }

    // for discriminated enums without explicit mapping, check if the variant name
    // matches what serde would use by default
    if discriminator.is_some() {
        if let TypeRef::Named(ref_name) = &variant.type_ref {
            if *ref_name != variant_ident.to_string() {
                return quote! { #[serde(rename = #ref_name)] };
            }
        }
    }

    quote! {}
}

fn generate_string_enum(name: &str, def: &StringEnumDef, doc: TokenStream) -> TokenStream {
    let enum_ident = to_pascal_ident(name);

    let variants: Vec<TokenStream> = def
        .values
        .iter()
        .map(|value| {
            let variant_ident = to_pascal_ident(value);
            let rename_attr = if variant_ident.to_string() != *value {
                quote! { #[serde(rename = #value)] }
            } else {
                quote! {}
            };
            quote! {
                #rename_attr
                #variant_ident,
            }
        })
        .collect();

    quote! {
        #doc
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum #enum_ident {
            #(#variants)*
        }
    }
}

fn generate_alias(name: &str, type_ref: &TypeRef, doc: TokenStream) -> TokenStream {
    let alias_ident = to_pascal_ident(name);
    let aliased_type = type_ref_to_tokens(type_ref);
    quote! {
        #doc
        pub type #alias_ident = #aliased_type;
    }
}

fn generate_doc_comment(description: &Option<String>) -> TokenStream {
    match description {
        Some(desc) => quote! { #[doc = #desc] },
        None => quote! {},
    }
}

fn type_ref_to_tokens(type_ref: &TypeRef) -> TokenStream {
    match type_ref {
        TypeRef::Named(name) => {
            let ident = to_pascal_ident(name);
            quote! { #ident }
        }
        TypeRef::Primitive(p) => primitive_to_tokens(p),
        TypeRef::Array(inner) => {
            let inner_tokens = type_ref_to_tokens(inner);
            quote! { Vec<#inner_tokens> }
        }
        TypeRef::Option(inner) => {
            let inner_tokens = type_ref_to_tokens(inner);
            quote! { Option<#inner_tokens> }
        }
        TypeRef::Map(inner) => {
            let inner_tokens = type_ref_to_tokens(inner);
            quote! { std::collections::HashMap<String, #inner_tokens> }
        }
    }
}

fn primitive_to_tokens(p: &PrimitiveType) -> TokenStream {
    match p {
        PrimitiveType::String => quote! { String },
        PrimitiveType::Bool => quote! { bool },
        PrimitiveType::I32 => quote! { i32 },
        PrimitiveType::I64 => quote! { i64 },
        PrimitiveType::F32 => quote! { f32 },
        PrimitiveType::F64 => quote! { f64 },
    }
}

fn variant_name_for_type_ref(type_ref: &TypeRef) -> Ident {
    let name = match type_ref {
        TypeRef::Named(n) => n.clone(),
        TypeRef::Primitive(p) => match p {
            PrimitiveType::String => "String".to_string(),
            PrimitiveType::Bool => "Bool".to_string(),
            PrimitiveType::I32 => "I32".to_string(),
            PrimitiveType::I64 => "I64".to_string(),
            PrimitiveType::F32 => "F32".to_string(),
            PrimitiveType::F64 => "F64".to_string(),
        },
        TypeRef::Array(inner) => {
            let inner_name = variant_name_for_type_ref(inner);
            format!("Vec{inner_name}")
        }
        TypeRef::Option(inner) => return variant_name_for_type_ref(inner),
        TypeRef::Map(inner) => {
            let inner_name = variant_name_for_type_ref(inner);
            format!("Map{inner_name}")
        }
    };
    to_pascal_ident(&name)
}

fn to_snake_ident(s: &str) -> Ident {
    let snake = s.to_snake_case();
    if RUST_KEYWORDS.contains(&snake.as_str()) {
        format_ident!("r#{}", snake)
    } else {
        Ident::new(&snake, Span::call_site())
    }
}

fn to_pascal_ident(s: &str) -> Ident {
    let pascal = s.to_pascal_case();
    Ident::new(&pascal, Span::call_site())
}
