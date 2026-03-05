use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use orator_core::codegen::{
    generate_operations_tokens, generate_types_tokens, group_by_tag, status_code_variant_name,
    to_pascal_ident, to_snake_ident, type_ref_to_tokens,
};
use orator_core::ir::{
    HttpMethod, OperationIr, OperationResponse, ParamLocation, ResponseStatusCode, TypeDef,
};

/// The result of generating a complete API module.
///
/// Contains the formatted Rust source for each file in the module.
pub struct GeneratedModule {
    /// Schema types (structs, enums, type aliases).
    pub types: String,
    /// Response enums, params structs, and API traits.
    pub operations: String,
    /// IntoResponse impls, handler functions, and router functions.
    pub handlers: String,
}

impl GeneratedModule {
    /// Generate a `mod.rs` for direct file write (future CLI use).
    pub fn mod_file(&self) -> String {
        [
            "#[allow(dead_code)]",
            "mod types;",
            "#[allow(dead_code)]",
            "mod operations;",
            "#[allow(dead_code)]",
            "mod handlers;",
            "",
            "pub use types::*;",
            "pub use operations::*;",
            "pub use handlers::*;",
            "",
        ]
        .join("\n")
    }

    /// Generate a bridge entry file for `build.rs` (`include!`-based).
    pub fn build_rs_entry(&self) -> String {
        [
            "#[allow(dead_code)]",
            "mod types {",
            r#"    include!(concat!(env!("OUT_DIR"), "/api/types.rs"));"#,
            "}",
            "",
            "#[allow(dead_code)]",
            "mod operations {",
            "    use super::types::*;",
            r#"    include!(concat!(env!("OUT_DIR"), "/api/operations.rs"));"#,
            "}",
            "",
            "#[allow(dead_code)]",
            "mod handlers {",
            "    use super::types::*;",
            "    use super::operations::*;",
            r#"    include!(concat!(env!("OUT_DIR"), "/api/handlers.rs"));"#,
            "}",
            "",
            "pub use types::*;",
            "pub use operations::*;",
            "pub use handlers::*;",
            "",
        ]
        .join("\n")
    }
}

fn format_tokens(tokens: Vec<TokenStream>) -> String {
    let file_tokens = quote! { #(#tokens)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    prettyplease::unparse(&syntax_tree)
}

/// Generate a complete API module from type definitions and operations.
pub fn generate(
    types: &[TypeDef],
    operations: &[OperationIr],
    default_tag: &str,
) -> GeneratedModule {
    let types_code = format_tokens(generate_types_tokens(types));
    let operations_code = format_tokens(generate_operations_tokens(operations, default_tag));
    let handlers_code = format_tokens(generate_axum_handlers_tokens(operations, default_tag));

    GeneratedModule {
        types: types_code,
        operations: operations_code,
        handlers: handlers_code,
    }
}

/// Generate token streams for axum handler functions, `IntoResponse` impls, and router functions.
pub fn generate_axum_handlers_tokens(
    operations: &[OperationIr],
    default_tag: &str,
) -> Vec<TokenStream> {
    let grouped = group_by_tag(operations, default_tag);

    let mut all_items = Vec::new();

    for (tag, ops) in &grouped {
        for op in ops {
            all_items.push(generate_into_response_impl(op));
            all_items.push(generate_handler_fn(op));
        }
        all_items.push(generate_router_fn(tag, ops));
    }

    all_items
}

/// Generate axum handler functions, `IntoResponse` impls, and router functions
/// for the given operations.
pub fn generate_axum_handlers(operations: &[OperationIr], default_tag: &str) -> String {
    let items = generate_axum_handlers_tokens(operations, default_tag);
    let file_tokens = quote! { #(#items)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    prettyplease::unparse(&syntax_tree)
}

fn status_code_to_tokens(response: &OperationResponse) -> TokenStream {
    match &response.status_code {
        ResponseStatusCode::Default => quote! { http::StatusCode::INTERNAL_SERVER_ERROR },
        ResponseStatusCode::Code(code) => {
            let constant = match code {
                200 => Some("OK"),
                201 => Some("CREATED"),
                202 => Some("ACCEPTED"),
                204 => Some("NO_CONTENT"),
                301 => Some("MOVED_PERMANENTLY"),
                304 => Some("NOT_MODIFIED"),
                400 => Some("BAD_REQUEST"),
                401 => Some("UNAUTHORIZED"),
                403 => Some("FORBIDDEN"),
                404 => Some("NOT_FOUND"),
                405 => Some("METHOD_NOT_ALLOWED"),
                409 => Some("CONFLICT"),
                410 => Some("GONE"),
                422 => Some("UNPROCESSABLE_ENTITY"),
                429 => Some("TOO_MANY_REQUESTS"),
                500 => Some("INTERNAL_SERVER_ERROR"),
                502 => Some("BAD_GATEWAY"),
                503 => Some("SERVICE_UNAVAILABLE"),
                _ => None,
            };
            if let Some(name) = constant {
                let ident = format_ident!("{}", name);
                quote! { http::StatusCode::#ident }
            } else {
                let code_lit = *code;
                quote! { http::StatusCode::from_u16(#code_lit).unwrap() }
            }
        }
    }
}

fn generate_into_response_impl(op: &OperationIr) -> TokenStream {
    let enum_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

    let arms: Vec<TokenStream> = op
        .responses
        .iter()
        .map(|resp| {
            let variant_name = status_code_variant_name(resp);
            let variant_ident = to_pascal_ident(&variant_name);
            let status = status_code_to_tokens(resp);

            if resp.body.is_some() {
                quote! {
                    Self::#variant_ident(body) => (#status, axum::Json(body)).into_response(),
                }
            } else {
                quote! {
                    Self::#variant_ident => #status.into_response(),
                }
            }
        })
        .collect();

    quote! {
        impl axum::response::IntoResponse for #enum_ident {
            fn into_response(self) -> axum::response::Response {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

fn generate_handler_fn(op: &OperationIr) -> TokenStream {
    let handler_ident = to_snake_ident(&format!("handle_{}", op.operation_id));
    let trait_ident = to_pascal_ident(&format!("{}Api", op.tag.as_deref().unwrap_or("Default")));
    let method_ident = to_snake_ident(&op.operation_id);
    let response_ident = to_pascal_ident(&format!("{}Response", op.operation_id));
    let params_ident = to_pascal_ident(&format!("{}Params", op.operation_id));

    // collect path parameters
    let path_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Path)
        .collect();

    // build function parameters
    let mut fn_params = Vec::new();

    // state is always first
    fn_params.push(quote! {
        axum::extract::State(api): axum::extract::State<std::sync::Arc<T>>
    });
    // ctx is second
    fn_params.push(quote! { ctx: Ctx });

    // path extractor after ctx
    if path_params.len() == 1 {
        let p = &path_params[0];
        let name = to_snake_ident(&p.name);
        let ty = type_ref_to_tokens(&p.type_ref);
        fn_params.push(quote! {
            axum::extract::Path(#name): axum::extract::Path<#ty>
        });
    } else if path_params.len() > 1 {
        let names: Vec<_> = path_params
            .iter()
            .map(|p| to_snake_ident(&p.name))
            .collect();
        let types: Vec<_> = path_params
            .iter()
            .map(|p| type_ref_to_tokens(&p.type_ref))
            .collect();
        fn_params.push(quote! {
            axum::extract::Path((#(#names),*)): axum::extract::Path<(#(#types),*)>
        });
    }

    // request body is last
    if let Some(body) = &op.request_body {
        let body_type = type_ref_to_tokens(&body.type_ref);
        fn_params.push(quote! {
            axum::Json(body): axum::Json<#body_type>
        });
    }

    // build params struct construction
    let has_fields = !op.parameters.is_empty() || op.request_body.is_some();
    let params_expr = if has_fields {
        let mut field_inits = Vec::new();
        for param in &op.parameters {
            let name = to_snake_ident(&param.name);
            field_inits.push(quote! { #name });
        }
        if op.request_body.is_some() {
            field_inits.push(quote! { body });
        }
        quote! { #params_ident { #(#field_inits),* } }
    } else {
        quote! { #params_ident }
    };

    quote! {
        async fn #handler_ident<T, Ctx>(
            #(#fn_params),*
        ) -> Result<#response_ident, T::Error>
        where
            T: #trait_ident<Ctx>,
        {
            api.#method_ident(ctx, #params_expr).await
        }
    }
}

fn http_method_to_routing_fn(method: &HttpMethod) -> proc_macro2::Ident {
    match method {
        HttpMethod::Get => format_ident!("get"),
        HttpMethod::Post => format_ident!("post"),
        HttpMethod::Put => format_ident!("put"),
        HttpMethod::Patch => format_ident!("patch"),
        HttpMethod::Delete => format_ident!("delete"),
        HttpMethod::Head => format_ident!("head"),
        HttpMethod::Options => format_ident!("options"),
        HttpMethod::Trace => format_ident!("trace"),
    }
}

fn generate_router_fn(tag: &str, operations: &[&OperationIr]) -> TokenStream {
    let router_ident = to_snake_ident(&format!("{tag}_router"));
    let trait_ident = to_pascal_ident(&format!("{tag}Api"));

    // Group operations by path, preserving order
    let mut path_groups: BTreeMap<&str, Vec<&OperationIr>> = BTreeMap::new();
    for op in operations {
        path_groups.entry(&op.path).or_default().push(op);
    }

    let route_calls: Vec<TokenStream> = path_groups
        .iter()
        .map(|(path, ops)| {
            let mut method_chain = Vec::new();
            for (i, op) in ops.iter().enumerate() {
                let routing_fn = http_method_to_routing_fn(&op.method);
                let handler_ident = to_snake_ident(&format!("handle_{}", op.operation_id));
                if i == 0 {
                    method_chain.push(quote! {
                        axum::routing::#routing_fn(#handler_ident::<T, Ctx>)
                    });
                } else {
                    method_chain.push(quote! {
                        .#routing_fn(#handler_ident::<T, Ctx>)
                    });
                }
            }
            quote! {
                .route(#path, #(#method_chain)*)
            }
        })
        .collect();

    quote! {
        pub fn #router_ident<T, Ctx>(api: std::sync::Arc<T>) -> axum::Router
        where
            T: #trait_ident<Ctx>,
            T::Error: axum::response::IntoResponse,
            Ctx: axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
        {
            axum::Router::new()
                #(#route_calls)*
                .with_state(api)
        }
    }
}
