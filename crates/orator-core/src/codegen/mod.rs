mod operations;
mod schemas;

pub use operations::{
    PARAM_LOCATIONS, generate_operations, generate_operations_tokens, group_by_tag,
    location_arg_name, location_suffix, status_code_variant_name,
};
pub use schemas::{generate_types, generate_types_tokens};

pub use crate::config::Config;

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use crate::ir::{PrimitiveType, TypeRef};

pub const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

pub fn to_snake_ident(s: &str) -> Ident {
    let snake = s.to_snake_case();
    if RUST_KEYWORDS.contains(&snake.as_str()) {
        format_ident!("r#{}", snake)
    } else {
        Ident::new(&snake, Span::call_site())
    }
}

pub fn to_pascal_ident(s: &str) -> Ident {
    let pascal = s.to_pascal_case();
    Ident::new(&pascal, Span::call_site())
}

pub fn type_ref_to_tokens(type_ref: &TypeRef) -> TokenStream {
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

pub fn primitive_to_tokens(p: &PrimitiveType) -> TokenStream {
    match p {
        PrimitiveType::String => quote! { String },
        PrimitiveType::Bool => quote! { bool },
        PrimitiveType::I32 => quote! { i32 },
        PrimitiveType::I64 => quote! { i64 },
        PrimitiveType::F32 => quote! { f32 },
        PrimitiveType::F64 => quote! { f64 },
        PrimitiveType::Bytes => quote! { orator_axum::bytes::Bytes },
    }
}

pub fn generate_doc_comment(description: &Option<String>) -> TokenStream {
    match description {
        Some(desc) => {
            let doc = format!(" {desc}");
            quote! { #[doc = #doc] }
        }
        None => quote! {},
    }
}
