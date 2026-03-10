use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use orator_core::codegen::{
    PARAM_LOCATIONS, generate_operations_tokens, generate_types_tokens, group_by_tag,
    location_suffix, status_code_variant_name, to_pascal_ident, to_snake_ident, type_ref_to_tokens,
};
pub use orator_core::config::Config;
use orator_core::ir::{
    HttpMethod, OperationIr, OperationParam, OperationResponse, ParamLocation, PrimitiveType,
    ResponseStatusCode, TypeDef, TypeRef,
};
use orator_core::lower::{lower_operations, lower_schemas};

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

    /// Write the module files to `OUT_DIR` for use from a `build.rs` script.
    ///
    /// Creates `api/types.rs`, `api/operations.rs`, `api/handlers.rs`, and
    /// a bridge `api.rs` entry file in the given output directory.
    pub fn write_to_out_dir(&self, out_dir: &Path) -> io::Result<()> {
        let api_dir = out_dir.join("api");
        fs::create_dir_all(&api_dir)?;
        fs::write(api_dir.join("types.rs"), &self.types)?;
        fs::write(api_dir.join("operations.rs"), &self.operations)?;
        fs::write(api_dir.join("handlers.rs"), &self.handlers)?;
        fs::write(out_dir.join("api.rs"), self.build_rs_entry())?;
        Ok(())
    }

    /// Write the module files directly into the given directory.
    ///
    /// Creates `types.rs`, `operations.rs`, `handlers.rs`, and `mod.rs` in the
    /// target directory. Intended for CLI use where files live in `src/`.
    pub fn write_to_dir(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("types.rs"), &self.types)?;
        fs::write(dir.join("operations.rs"), &self.operations)?;
        fs::write(dir.join("handlers.rs"), &self.handlers)?;
        fs::write(dir.join("mod.rs"), self.mod_file())?;
        Ok(())
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
    config: &Config,
) -> GeneratedModule {
    let types_code = format_tokens(generate_types_tokens(types));
    let operations_code =
        format_tokens(generate_operations_tokens(operations, default_tag, config));
    let handlers_code = format_tokens(generate_axum_handlers_tokens(
        operations,
        default_tag,
        config,
    ));

    GeneratedModule {
        types: types_code,
        operations: operations_code,
        handlers: handlers_code,
    }
}

/// Generate and write a complete API module from an OpenAPI spec file.
///
/// Intended to be called from a `build.rs` script. Reads and parses the spec,
/// generates the module files, writes them to `OUT_DIR`, and emits
/// `cargo::rerun-if-changed` for the spec file.
///
/// The user's `src/api.rs` should contain:
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/api.rs"));
/// ```
pub fn build(spec_path: impl AsRef<Path>) {
    build_with_config(spec_path, &Config::default());
}

pub fn build_with_config(spec_path: impl AsRef<Path>, config: &Config) {
    let spec_path = spec_path.as_ref();
    println!("cargo::rerun-if-changed={}", spec_path.display());

    let yaml = fs::read_to_string(spec_path).expect("failed to read OpenAPI spec");
    let spec = oas3::from_yaml(&yaml).expect("failed to parse OpenAPI spec");

    let types = lower_schemas(&spec).expect("failed to lower schemas");
    let ops = lower_operations(&spec).expect("failed to lower operations");

    let module = generate(&types, &ops, &spec.info.title, config);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    module
        .write_to_out_dir(Path::new(&out_dir))
        .expect("failed to write generated module");
}

/// Generate token streams for axum handler functions, `IntoResponse` impls, and router functions.
pub fn generate_axum_handlers_tokens(
    operations: &[OperationIr],
    default_tag: &str,
    config: &Config,
) -> Vec<TokenStream> {
    let grouped = group_by_tag(operations, default_tag);

    let mut all_items = Vec::new();

    for (tag, ops) in &grouped {
        for op in ops {
            all_items.push(generate_into_response_impl(op));
            all_items.push(generate_handler_fn(op, config));
        }
        all_items.push(generate_router_fn(tag, ops));
    }

    all_items.push(generate_api_builder(&grouped));

    all_items
}

/// Generate axum handler functions, `IntoResponse` impls, and router functions
/// for the given operations.
pub fn generate_axum_handlers(
    operations: &[OperationIr],
    default_tag: &str,
    config: &Config,
) -> String {
    let items = generate_axum_handlers_tokens(operations, default_tag, config);
    let file_tokens = quote! { #(#items)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    prettyplease::unparse(&syntax_tree)
}

fn status_code_to_tokens(response: &OperationResponse) -> TokenStream {
    match &response.status_code {
        ResponseStatusCode::Default => {
            quote! { orator_axum::http::StatusCode::INTERNAL_SERVER_ERROR }
        }
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
                quote! { orator_axum::http::StatusCode::#ident }
            } else {
                let code_lit = *code;
                quote! { orator_axum::http::StatusCode::from_u16(#code_lit).unwrap() }
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
                    Self::#variant_ident(body) => (#status, orator_axum::axum::Json(body)).into_response(),
                }
            } else {
                quote! {
                    Self::#variant_ident => #status.into_response(),
                }
            }
        })
        .collect();

    quote! {
        impl orator_axum::axum::response::IntoResponse for #enum_ident {
            fn into_response(self) -> orator_axum::axum::response::Response {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

fn generate_header_extraction(
    op: &OperationIr,
    header_params: &[&OperationParam],
) -> (TokenStream, TokenStream) {
    let header_struct = to_pascal_ident(&format!(
        "{}{}",
        op.operation_id,
        location_suffix(&ParamLocation::Header)
    ));

    let field_inits: Vec<TokenStream> = header_params
        .iter()
        .map(|param| {
            let field_name = to_snake_ident(&param.name);
            let header_name = &param.name;
            let is_string = matches!(&param.type_ref, TypeRef::Primitive(PrimitiveType::String));

            let value_expr = if is_string {
                quote! {
                    .to_str()
                    .expect(concat!("non-ASCII header value: ", #header_name))
                    .to_owned()
                }
            } else {
                let ty = type_ref_to_tokens(&param.type_ref);
                quote! {
                    .to_str()
                    .expect(concat!("non-ASCII header value: ", #header_name))
                    .parse::<#ty>()
                    .expect(concat!("invalid header value: ", #header_name))
                }
            };

            if param.required {
                quote! {
                    #field_name: headers
                        .get(#header_name)
                        .expect(concat!("missing required header: ", #header_name))
                        #value_expr,
                }
            } else {
                quote! {
                    #field_name: headers
                        .get(#header_name)
                        .map(|v| v #value_expr),
                }
            }
        })
        .collect();

    let fn_param = quote! { headers: orator_axum::axum::http::HeaderMap };
    let let_block = quote! {
        let header = #header_struct {
            #(#field_inits)*
        };
    };

    (fn_param, let_block)
}

fn generate_cookie_extraction(
    op: &OperationIr,
    cookie_params: &[&OperationParam],
) -> (TokenStream, TokenStream) {
    let cookie_struct = to_pascal_ident(&format!(
        "{}{}",
        op.operation_id,
        location_suffix(&ParamLocation::Cookie)
    ));

    let field_inits: Vec<TokenStream> = cookie_params
        .iter()
        .map(|param| {
            let field_name = to_snake_ident(&param.name);
            let cookie_name = &param.name;
            let is_string = matches!(&param.type_ref, TypeRef::Primitive(PrimitiveType::String));

            let value_expr = if is_string {
                quote! {
                    .value()
                    .to_owned()
                }
            } else {
                let ty = type_ref_to_tokens(&param.type_ref);
                quote! {
                    .value()
                    .parse::<#ty>()
                    .expect(concat!("invalid cookie value: ", #cookie_name))
                }
            };

            if param.required {
                quote! {
                    #field_name: jar
                        .get(#cookie_name)
                        .expect(concat!("missing required cookie: ", #cookie_name))
                        #value_expr,
                }
            } else {
                quote! {
                    #field_name: jar
                        .get(#cookie_name)
                        .map(|c| c #value_expr),
                }
            }
        })
        .collect();

    let fn_param = quote! { jar: orator_axum::axum_extra::extract::CookieJar };
    let let_block = quote! {
        let cookie = #cookie_struct {
            #(#field_inits)*
        };
    };

    (fn_param, let_block)
}

fn generate_handler_fn(op: &OperationIr, config: &Config) -> TokenStream {
    let handler_ident = to_snake_ident(&format!("handle_{}", op.operation_id));
    let trait_ident = to_pascal_ident(&format!("{}Api", op.tag.as_deref().unwrap_or("Default")));
    let method_ident = to_snake_ident(&op.operation_id);
    let response_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

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
        orator_axum::axum::extract::State(api): orator_axum::axum::extract::State<std::sync::Arc<T>>
    });
    // ctx is second
    fn_params.push(quote! { ctx: Ctx });

    // path extractor after ctx
    if path_params.len() == 1 {
        let p = &path_params[0];
        let name = to_snake_ident(&p.name);
        let ty = type_ref_to_tokens(&p.type_ref);
        fn_params.push(quote! {
            orator_axum::axum::extract::Path(#name): orator_axum::axum::extract::Path<#ty>
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
            orator_axum::axum::extract::Path((#(#names),*)): orator_axum::axum::extract::Path<(#(#types),*)>
        });
    }

    // query extractor
    let query_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Query)
        .collect();
    if !query_params.is_empty() {
        let query_struct = to_pascal_ident(&format!(
            "{}{}",
            op.operation_id,
            location_suffix(&ParamLocation::Query)
        ));
        fn_params.push(quote! {
            orator_axum::axum::extract::Query(query): orator_axum::axum::extract::Query<#query_struct>
        });
    }

    // header extractor
    let header_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Header)
        .filter(|_| config.is_location_enabled(&ParamLocation::Header))
        .collect();
    let header_extraction = if !header_params.is_empty() {
        let (fn_param, let_block) = generate_header_extraction(op, &header_params);
        fn_params.push(fn_param);
        Some(let_block)
    } else {
        None
    };

    // cookie extractor
    let cookie_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Cookie)
        .filter(|_| config.is_location_enabled(&ParamLocation::Cookie))
        .collect();
    let cookie_extraction = if !cookie_params.is_empty() {
        let (fn_param, let_block) = generate_cookie_extraction(op, &cookie_params);
        fn_params.push(fn_param);
        Some(let_block)
    } else {
        None
    };

    // request body is last
    if let Some(body) = &op.request_body {
        let body_type = type_ref_to_tokens(&body.type_ref);
        fn_params.push(quote! {
            orator_axum::axum::Json(body): orator_axum::axum::Json<#body_type>
        });
    }

    // build trait method call arguments
    let mut call_args = vec![quote! { ctx }];

    for location in PARAM_LOCATIONS {
        if !config.is_location_enabled(location) {
            continue;
        }

        let params_for_loc: Vec<_> = op
            .parameters
            .iter()
            .filter(|p| &p.location == location)
            .collect();

        if params_for_loc.is_empty() {
            continue;
        }

        if *location == ParamLocation::Query {
            call_args.push(quote! { query });
        } else if *location == ParamLocation::Header {
            call_args.push(quote! { header });
        } else if *location == ParamLocation::Cookie {
            call_args.push(quote! { cookie });
        } else {
            let params_ident =
                to_pascal_ident(&format!("{}{}", op.operation_id, location_suffix(location)));
            let field_inits: Vec<_> = params_for_loc
                .iter()
                .map(|param| to_snake_ident(&param.name))
                .map(|name| quote! { #name })
                .collect();
            let params_expr = quote! { #params_ident { #(#field_inits),* } };
            call_args.push(params_expr);
        }
    }

    if op.request_body.is_some() {
        call_args.push(quote! { body });
    }

    let method_call = quote! { api.#method_ident(#(#call_args),*).await };

    quote! {
        async fn #handler_ident<T, Ctx>(
            #(#fn_params),*
        ) -> Result<#response_ident, T::Error>
        where
            T: #trait_ident<Ctx>,
        {
            #header_extraction
            #cookie_extraction
            #method_call
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

fn generate_api_builder(tags: &BTreeMap<String, Vec<&OperationIr>>) -> TokenStream {
    if tags.is_empty() {
        return quote! {};
    }

    let tag_names: Vec<&String> = tags.keys().collect();

    struct TagInfo {
        wrapper_ident: proc_macro2::Ident,
        state_ident: proc_macro2::Ident,
        method_ident: proc_macro2::Ident,
        trait_ident: proc_macro2::Ident,
        router_fn_ident: proc_macro2::Ident,
    }

    let infos: Vec<TagInfo> = tag_names
        .iter()
        .map(|tag| TagInfo {
            wrapper_ident: to_pascal_ident(&format!("{tag}Router")),
            state_ident: to_pascal_ident(&format!("{tag}State")),
            method_ident: to_snake_ident(tag),
            trait_ident: to_pascal_ident(&format!("{tag}Api")),
            router_fn_ident: to_snake_ident(&format!("{tag}_router")),
        })
        .collect();

    let markers = quote! {
        pub struct Missing;
        pub struct Registered;
    };

    // per-tag typed router wrappers
    let wrappers: Vec<TokenStream> = infos
        .iter()
        .map(|info| {
            let wrapper = &info.wrapper_ident;
            let trait_ident = &info.trait_ident;
            let router_fn = &info.router_fn_ident;

            quote! {
                pub struct #wrapper(orator_axum::axum::Router);

                impl #wrapper {
                    pub fn new<T, Ctx>(api: std::sync::Arc<T>) -> Self
                    where
                        T: #trait_ident<Ctx>,
                        T::Error: orator_axum::axum::response::IntoResponse,
                        Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
                    {
                        Self(#router_fn(api))
                    }

                    pub fn layer<L>(self, layer: L) -> Self
                    where
                        L: orator_axum::tower::Layer<orator_axum::axum::routing::Route> + Clone + Send + Sync + 'static,
                        L::Service: orator_axum::tower::Service<orator_axum::axum::http::Request<orator_axum::axum::body::Body>>
                            + Clone + Send + Sync + 'static,
                        <L::Service as orator_axum::tower::Service<orator_axum::axum::http::Request<orator_axum::axum::body::Body>>>::Response:
                            orator_axum::axum::response::IntoResponse + 'static,
                        <L::Service as orator_axum::tower::Service<orator_axum::axum::http::Request<orator_axum::axum::body::Body>>>::Error:
                            Into<std::convert::Infallible> + 'static,
                        <L::Service as orator_axum::tower::Service<orator_axum::axum::http::Request<orator_axum::axum::body::Body>>>::Future:
                            Send + 'static,
                    {
                        Self(self.0.layer(layer))
                    }
                }
            }
        })
        .collect();

    // ApiBuilder struct
    let state_idents: Vec<&proc_macro2::Ident> = infos.iter().map(|i| &i.state_ident).collect();

    let builder_struct = quote! {
        pub struct ApiBuilder<#(#state_idents = Missing),*> {
            router: orator_axum::axum::Router,
            _phantom: std::marker::PhantomData<(#(#state_idents),*)>,
        }
    };

    // new() impl on all-defaults type
    let new_impl = quote! {
        impl ApiBuilder {
            pub fn new() -> Self {
                Self {
                    router: orator_axum::axum::Router::new(),
                    _phantom: std::marker::PhantomData,
                }
            }
        }
    };

    // per-tag registration methods
    let registration_impls: Vec<TokenStream> = infos
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let wrapper = &info.wrapper_ident;
            let method = &info.method_ident;

            let impl_generics: Vec<&proc_macro2::Ident> = infos
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != idx)
                .map(|(_, i)| &i.state_ident)
                .collect();

            let input_params: Vec<TokenStream> = infos
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    if i == idx {
                        quote! { Missing }
                    } else {
                        let ident = &info.state_ident;
                        quote! { #ident }
                    }
                })
                .collect();

            let output_params: Vec<TokenStream> = infos
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    if i == idx {
                        quote! { Registered }
                    } else {
                        let ident = &info.state_ident;
                        quote! { #ident }
                    }
                })
                .collect();

            quote! {
                impl<#(#impl_generics),*> ApiBuilder<#(#input_params),*> {
                    pub fn #method(self, router: #wrapper) -> ApiBuilder<#(#output_params),*> {
                        ApiBuilder {
                            router: self.router.merge(router.0),
                            _phantom: std::marker::PhantomData,
                        }
                    }
                }
            }
        })
        .collect();

    // build() impl for all Registered state
    let all_registered: Vec<TokenStream> = infos.iter().map(|_| quote! { Registered }).collect();
    let build_impl = quote! {
        impl ApiBuilder<#(#all_registered),*> {
            pub fn build(self) -> orator_axum::axum::Router {
                self.router
            }
        }
    };

    quote! {
        #markers
        #(#wrappers)*
        #builder_struct
        #new_impl
        #(#registration_impls)*
        #build_impl
    }
}

fn generate_router_fn(tag: &str, operations: &[&OperationIr]) -> TokenStream {
    let router_ident = to_snake_ident(&format!("{tag}_router"));
    let trait_ident = to_pascal_ident(&format!("{tag}Api"));

    // group operations by path, preserving order
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
                        orator_axum::axum::routing::#routing_fn(#handler_ident::<T, Ctx>)
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
        pub fn #router_ident<T, Ctx>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
        where
            T: #trait_ident<Ctx>,
            T::Error: orator_axum::axum::response::IntoResponse,
            Ctx: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
        {
            orator_axum::axum::Router::new()
                #(#route_calls)*
                .with_state(api)
        }
    }
}
