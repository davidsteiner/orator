use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{OperationIr, OperationResponse, ResponseStatusCode};

use super::{to_pascal_ident, to_snake_ident, type_ref_to_qualified_tokens as type_ref_to_tokens};

/// Generate Rust source code for a list of operations.
pub fn generate_operations(operations: &[OperationIr], default_tag: &str) -> String {
    let grouped = group_by_tag(operations, default_tag);

    let mut all_items = Vec::new();

    for (tag, ops) in &grouped {
        for op in ops {
            all_items.push(generate_response_enum(op));
            all_items.push(generate_params_struct(op));
        }
        all_items.push(generate_api_trait(tag, ops));
    }

    let file_tokens = quote! { #(#all_items)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    prettyplease::unparse(&syntax_tree)
}

pub fn group_by_tag<'a>(
    operations: &'a [OperationIr],
    default_tag: &str,
) -> BTreeMap<String, Vec<&'a OperationIr>> {
    let mut groups: BTreeMap<String, Vec<&'a OperationIr>> = BTreeMap::new();
    for op in operations {
        let tag = op.tag.as_deref().unwrap_or(default_tag);
        groups.entry(tag.to_string()).or_default().push(op);
    }
    groups
}

pub fn status_code_variant_name(response: &OperationResponse) -> String {
    match &response.status_code {
        ResponseStatusCode::Default => "Default".to_string(),
        ResponseStatusCode::Code(code) => match code {
            200 => "Ok".to_string(),
            201 => "Created".to_string(),
            202 => "Accepted".to_string(),
            204 => "NoContent".to_string(),
            301 => "MovedPermanently".to_string(),
            304 => "NotModified".to_string(),
            400 => "BadRequest".to_string(),
            401 => "Unauthorized".to_string(),
            403 => "Forbidden".to_string(),
            404 => "NotFound".to_string(),
            405 => "MethodNotAllowed".to_string(),
            409 => "Conflict".to_string(),
            410 => "Gone".to_string(),
            422 => "UnprocessableEntity".to_string(),
            429 => "TooManyRequests".to_string(),
            500 => "InternalServerError".to_string(),
            502 => "BadGateway".to_string(),
            503 => "ServiceUnavailable".to_string(),
            other => format!("Status{other}"),
        },
    }
}

fn generate_response_enum(op: &OperationIr) -> TokenStream {
    let enum_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

    let variants: Vec<TokenStream> = op
        .responses
        .iter()
        .map(|resp| {
            let variant_name = status_code_variant_name(resp);
            let variant_ident = to_pascal_ident(&variant_name);

            if let Some(body) = &resp.body {
                let body_type = type_ref_to_tokens(body);
                quote! { #variant_ident(#body_type), }
            } else {
                quote! { #variant_ident, }
            }
        })
        .collect();

    quote! {
        #[allow(dead_code)]
        pub enum #enum_ident {
            #(#variants)*
        }
    }
}

fn generate_params_struct(op: &OperationIr) -> TokenStream {
    let struct_ident = to_pascal_ident(&format!("{}Params", op.operation_id));

    let mut fields: Vec<TokenStream> = op
        .parameters
        .iter()
        .map(|param| {
            let field_ident = to_snake_ident(&param.name);
            let field_type = if param.required {
                type_ref_to_tokens(&param.type_ref)
            } else {
                let inner = type_ref_to_tokens(&param.type_ref);
                quote! { Option<#inner> }
            };
            quote! { pub #field_ident: #field_type, }
        })
        .collect();

    if let Some(body) = &op.request_body {
        let body_type = if body.required {
            type_ref_to_tokens(&body.type_ref)
        } else {
            let inner = type_ref_to_tokens(&body.type_ref);
            quote! { Option<#inner> }
        };
        fields.push(quote! { pub body: #body_type, });
    }

    if fields.is_empty() {
        quote! {
            #[allow(dead_code)]
            pub struct #struct_ident;
        }
    } else {
        quote! {
            #[allow(dead_code)]
            pub struct #struct_ident {
                #(#fields)*
            }
        }
    }
}

fn generate_api_trait(tag: &str, operations: &[&OperationIr]) -> TokenStream {
    let trait_ident = to_pascal_ident(&format!("{tag}Api"));

    let methods: Vec<TokenStream> = operations
        .iter()
        .map(|op| {
            let method_ident = to_snake_ident(&op.operation_id);
            let params_ident = to_pascal_ident(&format!("{}Params", op.operation_id));
            let response_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

            quote! {
                fn #method_ident(
                    &self,
                    ctx: Ctx,
                    params: #params_ident,
                ) -> impl std::future::Future<Output = Result<#response_ident, Self::Error>> + Send;
            }
        })
        .collect();

    quote! {
        pub trait #trait_ident<Ctx = ()>: Send + Sync + 'static {
            type Error: Send;

            #(#methods)*
        }
    }
}
