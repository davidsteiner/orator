use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use orator_core::codegen::{
    PARAM_LOCATIONS, SpecInfo, generate_operations_tokens, generate_types_tokens,
    generated_file_banner, generated_file_preamble, group_by_tag, location_suffix,
    multipart_body_struct_name, status_code_variant_name, to_pascal_ident, to_snake_ident,
    type_ref_to_tokens,
};
pub use orator_core::config::Config;
use orator_core::ir::{
    ContentType, HttpMethod, OperationIr, OperationParam, OperationResponse, ParamLocation,
    PrimitiveType, ResponseStatusCode, TypeDef, TypeRef,
};

/// Module prefix for schema types referenced from handlers.
fn types_prefix() -> TokenStream {
    quote! { super::types }
}

/// Module prefix for operation types (response enums, param structs, traits) referenced from handlers.
fn ops_prefix() -> TokenStream {
    quote! { super::operations }
}

/// The result of generating a complete API module.
///
/// Contains the formatted Rust source for each file in the module.
pub struct GeneratedModule {
    /// Banner comment prepended to each file.
    banner: String,
    /// Schema types (structs, enums, type aliases).
    pub types: String,
    /// Response enums, params structs, and API traits.
    pub operations: String,
    /// IntoResponse impls, handler functions, and router functions.
    pub handlers: String,
}

impl GeneratedModule {
    /// Generate a `mod.rs` for the module.
    pub fn mod_file(&self) -> String {
        format!(
            "{}pub mod types;\npub mod operations;\npub mod handlers;\n",
            self.banner,
        )
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
}

fn format_tokens(tokens: Vec<TokenStream>, banner: &str) -> String {
    let file_tokens = quote! { #(#tokens)* };
    let syntax_tree: syn::File =
        syn::parse2(file_tokens).expect("generated tokens should be valid syntax");
    format!("{}{}", banner, prettyplease::unparse(&syntax_tree))
}

/// Generate a complete API module from type definitions and operations.
pub fn generate(
    types: &[TypeDef],
    operations: &[OperationIr],
    default_tag: &str,
    config: &Config,
    spec_info: &SpecInfo,
) -> GeneratedModule {
    let banner = generated_file_banner(spec_info, env!("CARGO_PKG_VERSION"));
    let types_code = format_tokens(generate_types_tokens(types), &banner);
    let operations_code = format_tokens(
        generate_operations_tokens(operations, default_tag, config),
        &banner,
    );
    let handlers_code = format_tokens(
        generate_axum_handlers_tokens(operations, default_tag, config),
        &banner,
    );

    GeneratedModule {
        banner,
        types: types_code,
        operations: operations_code,
        handlers: handlers_code,
    }
}

/// Generate token streams for axum handler functions, `IntoResponse` impls, and router functions.
pub fn generate_axum_handlers_tokens(
    operations: &[OperationIr],
    default_tag: &str,
    config: &Config,
) -> Vec<TokenStream> {
    let grouped = group_by_tag(operations, default_tag);

    let mut all_items = vec![generated_file_preamble()];

    for (tag, ops) in &grouped {
        for op in ops {
            all_items.push(generate_into_response_impl(op));
            all_items.extend(generate_multipart_from_request_impl(op));
            let handler_output = generate_handler_fn(op, config);
            all_items.extend(handler_output.extractor_impls);
            all_items.push(handler_output.handler_fn);
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
            unreachable!("default responses use runtime status codes")
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
    let ops = ops_prefix();
    let enum_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

    let arms: Vec<TokenStream> = op
        .responses
        .iter()
        .map(|resp| {
            let variant_name = status_code_variant_name(resp);
            let variant_ident = to_pascal_ident(&variant_name);
            let is_default = matches!(&resp.status_code, ResponseStatusCode::Default);

            if is_default {
                if let Some(body) = &resp.body {
                    let response_expr = match &body.content_type {
                        ContentType::Json => {
                            quote! { (status, orator_axum::axum::Json(body)).into_response() }
                        }
                        ContentType::TextPlain | ContentType::OctetStream => {
                            quote! { (status, body).into_response() }
                        }
                        ContentType::FormUrlEncoded | ContentType::MultipartFormData => {
                            unreachable!()
                        }
                    };
                    quote! {
                        Self::#variant_ident(status, body) => #response_expr,
                    }
                } else {
                    quote! {
                        Self::#variant_ident(status) => status.into_response(),
                    }
                }
            } else {
                let status = status_code_to_tokens(resp);

                if let Some(body) = &resp.body {
                    let response_expr = match &body.content_type {
                        ContentType::Json => {
                            quote! { (#status, orator_axum::axum::Json(body)).into_response() }
                        }
                        ContentType::TextPlain | ContentType::OctetStream => {
                            quote! { (#status, body).into_response() }
                        }
                        ContentType::FormUrlEncoded | ContentType::MultipartFormData => {
                            unreachable!()
                        }
                    };
                    quote! {
                        Self::#variant_ident(body) => #response_expr,
                    }
                } else {
                    quote! {
                        Self::#variant_ident => #status.into_response(),
                    }
                }
            }
        })
        .collect();

    quote! {
        impl orator_axum::axum::response::IntoResponse for #ops::#enum_ident {
            fn into_response(self) -> orator_axum::axum::response::Response {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

fn generate_header_extractor(
    op: &OperationIr,
    header_params: &[&OperationParam],
) -> (TokenStream, TokenStream) {
    let ops = ops_prefix();
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
                    .map_err(|_| orator_axum::ParamRejection::non_ascii(
                        orator_axum::ParamLocation::Header, #header_name))?
                    .to_owned()
                }
            } else {
                let ty = type_ref_to_tokens(&param.type_ref, Some(&types_prefix()));
                quote! {
                    .to_str()
                    .map_err(|_| orator_axum::ParamRejection::non_ascii(
                        orator_axum::ParamLocation::Header, #header_name))?
                    .parse::<#ty>()
                    .map_err(|e| orator_axum::ParamRejection::invalid(
                        orator_axum::ParamLocation::Header, #header_name, e))?
                }
            };

            if param.required {
                quote! {
                    #field_name: headers
                        .get(#header_name)
                        .ok_or_else(|| orator_axum::ParamRejection::missing(
                            orator_axum::ParamLocation::Header, #header_name))?
                        #value_expr,
                }
            } else {
                quote! {
                    #field_name: match headers.get(#header_name) {
                        Some(v) => Some(v #value_expr),
                        None => None,
                    },
                }
            }
        })
        .collect();

    let fn_param = quote! { header: #ops::#header_struct };
    let impl_block = quote! {
        impl<S> orator_axum::axum::extract::FromRequestParts<S> for #ops::#header_struct
        where
            S: Send + Sync,
        {
            type Rejection = orator_axum::ParamRejection;

            async fn from_request_parts(
                parts: &mut orator_axum::http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                let headers = &parts.headers;
                Ok(Self {
                    #(#field_inits)*
                })
            }
        }
    };

    (fn_param, impl_block)
}

fn generate_cookie_extractor(
    op: &OperationIr,
    cookie_params: &[&OperationParam],
) -> (TokenStream, TokenStream) {
    let ops = ops_prefix();
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
                let ty = type_ref_to_tokens(&param.type_ref, Some(&types_prefix()));
                quote! {
                    .value()
                    .parse::<#ty>()
                    .map_err(|e| orator_axum::ParamRejection::invalid(
                        orator_axum::ParamLocation::Cookie, #cookie_name, e))?
                }
            };

            if param.required {
                quote! {
                    #field_name: jar
                        .get(#cookie_name)
                        .ok_or_else(|| orator_axum::ParamRejection::missing(
                            orator_axum::ParamLocation::Cookie, #cookie_name))?
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

    let fn_param = quote! { cookie: #ops::#cookie_struct };
    let impl_block = quote! {
        impl<S> orator_axum::axum::extract::FromRequestParts<S> for #ops::#cookie_struct
        where
            S: Send + Sync,
        {
            type Rejection = orator_axum::ParamRejection;

            async fn from_request_parts(
                parts: &mut orator_axum::http::request::Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                let jar = orator_axum::axum_extra::extract::CookieJar::from_request_parts(parts, state)
                    .await
                    .unwrap();
                Ok(Self {
                    #(#field_inits)*
                })
            }
        }
    };

    (fn_param, impl_block)
}

fn generate_multipart_from_request_impl(op: &OperationIr) -> Option<TokenStream> {
    let struct_name = multipart_body_struct_name(op)?;
    let body = op.request_body.as_ref()?;
    let fields = body.fields.as_ref()?;

    let ops = ops_prefix();
    let struct_ident = to_pascal_ident(&struct_name);

    // For each field, generate:
    // 1. A mutable local variable initialized to None
    // 2. A match arm in the field-name match that sets it
    // 3. A final unwrap/take for required vs optional

    let local_vars: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let var = to_snake_ident(&field.name);
            quote! { let mut #var = None; }
        })
        .collect();

    let match_arms: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name_str = &field.name;
            let var = to_snake_ident(&field.name);
            let is_bytes = matches!(&field.type_ref, TypeRef::Primitive(PrimitiveType::Bytes));

            let value_expr = if is_bytes {
                quote! { field.bytes().await.map_err(|e| {
                    orator_axum::axum::response::Response::builder()
                        .status(orator_axum::http::StatusCode::BAD_REQUEST)
                        .body(orator_axum::axum::body::Body::from(e.to_string()))
                        .unwrap()
                })? }
            } else {
                quote! { field.text().await.map_err(|e| {
                    orator_axum::axum::response::Response::builder()
                        .status(orator_axum::http::StatusCode::BAD_REQUEST)
                        .body(orator_axum::axum::body::Body::from(e.to_string()))
                        .unwrap()
                })? }
            };

            quote! {
                #field_name_str => {
                    #var = Some(#value_expr);
                }
            }
        })
        .collect();

    let field_inits: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let var = to_snake_ident(&field.name);
            let field_name_str = &field.name;
            if field.required {
                quote! {
                    #var: #var.ok_or_else(|| {
                        orator_axum::axum::response::Response::builder()
                            .status(orator_axum::http::StatusCode::BAD_REQUEST)
                            .body(orator_axum::axum::body::Body::from(
                                format!("missing required multipart field: {}", #field_name_str),
                            ))
                            .unwrap()
                    })?,
                }
            } else {
                quote! { #var, }
            }
        })
        .collect();

    Some(quote! {
        impl<S> orator_axum::axum::extract::FromRequest<S> for #ops::#struct_ident
        where
            S: Send + Sync,
        {
            type Rejection = orator_axum::axum::response::Response;

            async fn from_request(
                req: orator_axum::axum::http::Request<orator_axum::axum::body::Body>,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                let mut multipart = orator_axum::axum::extract::Multipart::from_request(req, state)
                    .await
                    .map_err(|e| {
                        use orator_axum::axum::response::IntoResponse;
                        e.into_response()
                    })?;

                #(#local_vars)*

                while let Some(field) = multipart.next_field().await.map_err(|e| {
                    orator_axum::axum::response::Response::builder()
                        .status(orator_axum::http::StatusCode::BAD_REQUEST)
                        .body(orator_axum::axum::body::Body::from(e.to_string()))
                        .unwrap()
                })? {
                    let name = field.name().unwrap_or_default().to_owned();
                    match name.as_str() {
                        #(#match_arms)*
                        _ => {}
                    }
                }

                Ok(Self {
                    #(#field_inits)*
                })
            }
        }
    })
}

/// Result of generating a handler function and its associated extractor impls.
struct HandlerOutput {
    handler_fn: TokenStream,
    extractor_impls: Vec<TokenStream>,
}

fn generate_handler_fn(op: &OperationIr, config: &Config) -> HandlerOutput {
    let ops = ops_prefix();
    let handler_ident = to_snake_ident(&format!("handle_{}", op.operation_id));
    let trait_ident = to_pascal_ident(&format!("{}Api", op.tag.as_deref().unwrap_or("Default")));
    let method_ident = to_snake_ident(&op.operation_id);
    let response_ident = to_pascal_ident(&format!("{}Response", op.operation_id));

    let mut extractor_impls = Vec::new();

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
    fn_params.push(quote! { ctx: T::RequestContext });

    // path extractor after ctx
    if path_params.len() == 1 {
        let p = &path_params[0];
        let name = to_snake_ident(&p.name);
        let ty = type_ref_to_tokens(&p.type_ref, Some(&types_prefix()));
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
            .map(|p| type_ref_to_tokens(&p.type_ref, Some(&types_prefix())))
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
            orator_axum::axum::extract::Query(query): orator_axum::axum::extract::Query<#ops::#query_struct>
        });
    }

    // header extractor
    let header_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Header)
        .filter(|_| config.is_location_enabled(&ParamLocation::Header))
        .collect();
    if !header_params.is_empty() {
        let (fn_param, impl_block) = generate_header_extractor(op, &header_params);
        fn_params.push(fn_param);
        extractor_impls.push(impl_block);
    }

    // cookie extractor
    let cookie_params: Vec<_> = op
        .parameters
        .iter()
        .filter(|p| p.location == ParamLocation::Cookie)
        .filter(|_| config.is_location_enabled(&ParamLocation::Cookie))
        .collect();
    if !cookie_params.is_empty() {
        let (fn_param, impl_block) = generate_cookie_extractor(op, &cookie_params);
        fn_params.push(fn_param);
        extractor_impls.push(impl_block);
    }

    // request body is last
    if let Some(body) = &op.request_body {
        let body_type = type_ref_to_tokens(&body.type_ref, Some(&types_prefix()));
        match (&body.content_type, body.required) {
            (ContentType::Json, true) => {
                fn_params.push(quote! {
                    orator_axum::axum::Json(body): orator_axum::axum::Json<#body_type>
                });
            }
            (ContentType::Json, false) => {
                fn_params.push(quote! {
                    body: Option<orator_axum::axum::Json<#body_type>>
                });
            }
            (ContentType::TextPlain, true) => {
                fn_params.push(quote! { body: String });
            }
            (ContentType::TextPlain, false) => {
                fn_params.push(quote! { body: Option<String> });
            }
            (ContentType::OctetStream, true) => {
                fn_params.push(quote! { body: orator_axum::axum::body::Bytes });
            }
            (ContentType::OctetStream, false) => {
                fn_params.push(quote! { body: Option<orator_axum::axum::body::Bytes> });
            }
            (ContentType::FormUrlEncoded, true) => {
                fn_params.push(quote! {
                    orator_axum::axum::Form(body): orator_axum::axum::Form<#body_type>
                });
            }
            (ContentType::FormUrlEncoded, false) => {
                fn_params.push(quote! {
                    body: Option<orator_axum::axum::Form<#body_type>>
                });
            }
            (ContentType::MultipartFormData, _) => {
                if let Some(struct_name) = multipart_body_struct_name(op) {
                    let body_struct = to_pascal_ident(&struct_name);
                    fn_params.push(quote! {
                        body: #ops::#body_struct
                    });
                } else {
                    fn_params.push(quote! {
                        body: orator_axum::axum::extract::Multipart
                    });
                }
            }
        }
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
            let params_expr = quote! { #ops::#params_ident { #(#field_inits),* } };
            call_args.push(params_expr);
        }
    }

    if let Some(body) = &op.request_body {
        if !body.required {
            match &body.content_type {
                ContentType::Json => {
                    call_args.push(quote! { body.map(|orator_axum::axum::Json(b)| b) });
                }
                ContentType::FormUrlEncoded => {
                    call_args.push(quote! { body.map(|orator_axum::axum::Form(b)| b) });
                }
                _ => call_args.push(quote! { body }),
            }
        } else {
            call_args.push(quote! { body });
        }
    }

    let method_call = quote! { api.#method_ident(#(#call_args),*).await };

    let handler_fn = quote! {
        async fn #handler_ident<T>(
            #(#fn_params),*
        ) -> Result<#ops::#response_ident, T::Error>
        where
            T: #ops::#trait_ident,
        {
            #method_call
        }
    };

    HandlerOutput {
        handler_fn,
        extractor_impls,
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

    let ops = ops_prefix();
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
                    pub fn new<T>(api: std::sync::Arc<T>) -> Self
                    where
                        T: #ops::#trait_ident,
                        T::Error: orator_axum::axum::response::IntoResponse,
                        T::RequestContext: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
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
    let ops = ops_prefix();
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
                        orator_axum::axum::routing::#routing_fn(#handler_ident::<T>)
                    });
                } else {
                    method_chain.push(quote! {
                        .#routing_fn(#handler_ident::<T>)
                    });
                }
            }
            quote! {
                .route(#path, #(#method_chain)*)
            }
        })
        .collect();

    quote! {
        pub fn #router_ident<T>(api: std::sync::Arc<T>) -> orator_axum::axum::Router
        where
            T: #ops::#trait_ident,
            T::Error: orator_axum::axum::response::IntoResponse,
            T::RequestContext: orator_axum::axum::extract::FromRequestParts<std::sync::Arc<T>> + Send + 'static,
        {
            orator_axum::axum::Router::new()
                #(#route_calls)*
                .with_state(api)
        }
    }
}
