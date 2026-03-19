use oas3::spec::{ObjectOrReference, Parameter, ParameterIn, Response};

use crate::error::Error;
use crate::ir::{
    ContentType, Field, HttpMethod, OperationIr, OperationParam, OperationResponse, ParamLocation,
    PrimitiveType, RequestBodyIr, ResponseBody, ResponseStatusCode, TypeRef,
};

use super::schemas::lower_type_ref;

pub fn lower_operations(spec: &oas3::Spec) -> Result<Vec<OperationIr>, Error> {
    let Some(paths) = &spec.paths else {
        return Ok(Vec::new());
    };

    let mut operations = Vec::new();

    for (path_str, path_item) in paths {
        let path_params = &path_item.parameters;

        let methods: [(HttpMethod, &Option<oas3::spec::Operation>); 8] = [
            (HttpMethod::Get, &path_item.get),
            (HttpMethod::Post, &path_item.post),
            (HttpMethod::Put, &path_item.put),
            (HttpMethod::Patch, &path_item.patch),
            (HttpMethod::Delete, &path_item.delete),
            (HttpMethod::Head, &path_item.head),
            (HttpMethod::Options, &path_item.options),
            (HttpMethod::Trace, &path_item.trace),
        ];

        for (method, maybe_op) in methods {
            let Some(op) = maybe_op else { continue };

            let operation_id =
                op.operation_id
                    .clone()
                    .ok_or_else(|| Error::MissingOperationId {
                        method: format!("{method:?}").to_uppercase(),
                        path: path_str.clone(),
                    })?;

            let tag = op.tags.first().cloned();

            let parameters = merge_and_lower_params(path_params, &op.parameters, spec)?;

            let request_body = lower_request_body(&operation_id, &op.request_body, spec)?;

            let responses = lower_responses(&op.responses, spec)?;

            let description = op.summary.clone().or(op.description.clone());

            operations.push(OperationIr {
                operation_id,
                description,
                method,
                path: path_str.clone(),
                tag,
                parameters,
                request_body,
                responses,
            });
        }
    }

    Ok(operations)
}

/// Merge path-level and operation-level parameters.
///
/// Operation params override path params when they share the same name and location.
fn merge_and_lower_params(
    path_params: &[ObjectOrReference<Parameter>],
    op_params: &[ObjectOrReference<Parameter>],
    spec: &oas3::Spec,
) -> Result<Vec<OperationParam>, Error> {
    let mut resolved: Vec<Parameter> = Vec::new();

    for p in path_params {
        resolved.push(resolve_parameter(p, spec)?);
    }

    for p in op_params {
        let param = resolve_parameter(p, spec)?;
        if let Some(existing) = resolved
            .iter_mut()
            .find(|r| r.name == param.name && r.location == param.location)
        {
            *existing = param;
        } else {
            resolved.push(param);
        }
    }

    resolved.iter().map(lower_parameter).collect()
}

fn resolve_parameter(
    param: &ObjectOrReference<Parameter>,
    spec: &oas3::Spec,
) -> Result<Parameter, Error> {
    match param {
        ObjectOrReference::Object(p) => Ok(p.clone()),
        ObjectOrReference::Ref { ref_path, .. } => {
            param.resolve(spec).map_err(|_| Error::UnresolvedRef {
                ref_path: ref_path.clone(),
            })
        }
    }
}

fn lower_parameter(param: &Parameter) -> Result<OperationParam, Error> {
    let location = match param.location {
        ParameterIn::Path => ParamLocation::Path,
        ParameterIn::Query => ParamLocation::Query,
        ParameterIn::Header => ParamLocation::Header,
        ParameterIn::Cookie => ParamLocation::Cookie,
    };

    let type_ref = match &param.schema {
        Some(schema_or_ref) => lower_type_ref(schema_or_ref)?,
        None => TypeRef::Primitive(crate::ir::PrimitiveType::String),
    };

    Ok(OperationParam {
        name: param.name.clone(),
        description: param.description.clone(),
        location,
        type_ref,
        required: param.required.unwrap_or(false),
    })
}

const REQUEST_CONTENT_TYPE_PRIORITY: &[(&str, ContentType)] = &[
    ("application/json", ContentType::Json),
    ("text/plain", ContentType::TextPlain),
    ("application/octet-stream", ContentType::OctetStream),
    (
        "application/x-www-form-urlencoded",
        ContentType::FormUrlEncoded,
    ),
    ("multipart/form-data", ContentType::MultipartFormData),
];

const RESPONSE_CONTENT_TYPE_PRIORITY: &[(&str, ContentType)] = &[
    ("application/json", ContentType::Json),
    ("text/plain", ContentType::TextPlain),
    ("application/octet-stream", ContentType::OctetStream),
];

fn lower_request_body(
    operation_id: &str,
    body: &Option<ObjectOrReference<oas3::spec::RequestBody>>,
    spec: &oas3::Spec,
) -> Result<Option<RequestBodyIr>, Error> {
    let Some(body_ref) = body else {
        return Ok(None);
    };

    let resolved = match body_ref {
        ObjectOrReference::Object(b) => b.clone(),
        ObjectOrReference::Ref { ref_path, .. } => {
            body_ref.resolve(spec).map_err(|_| Error::UnresolvedRef {
                ref_path: ref_path.clone(),
            })?
        }
    };

    let (content_type, media_type) = REQUEST_CONTENT_TYPE_PRIORITY
        .iter()
        .find_map(|(mime, ct)| resolved.content.get(*mime).map(|mt| (ct.clone(), mt)))
        .ok_or_else(|| {
            let found: Vec<_> = resolved.content.keys().cloned().collect();
            Error::UnsupportedRequestBodyMediaType {
                operation_id: operation_id.to_string(),
                media_types: found,
            }
        })?;

    let (type_ref, fields) = match &content_type {
        ContentType::TextPlain => (TypeRef::Primitive(PrimitiveType::String), None),
        ContentType::OctetStream => (TypeRef::Primitive(PrimitiveType::Bytes), None),
        ContentType::MultipartFormData => {
            let fields = lower_multipart_fields(&media_type.schema)?;
            (TypeRef::Primitive(PrimitiveType::String), Some(fields))
        }
        ContentType::Json | ContentType::FormUrlEncoded => {
            let tr = match &media_type.schema {
                Some(schema_or_ref) => lower_type_ref(schema_or_ref)?,
                None => TypeRef::Primitive(PrimitiveType::String),
            };
            (tr, None)
        }
    };

    Ok(Some(RequestBodyIr {
        content_type,
        type_ref,
        required: resolved.required.unwrap_or(false),
        fields,
    }))
}

fn lower_multipart_fields(
    schema: &Option<ObjectOrReference<oas3::spec::ObjectSchema>>,
) -> Result<Vec<Field>, Error> {
    let Some(schema_or_ref) = schema else {
        return Ok(Vec::new());
    };

    let obj = match schema_or_ref {
        ObjectOrReference::Object(s) => s,
        ObjectOrReference::Ref { ref_path, .. } => {
            return Err(Error::UnsupportedSchema {
                context: format!(
                    "multipart form-data with $ref schema not yet supported: {ref_path}"
                ),
            });
        }
    };

    let mut fields = Vec::new();
    for (prop_name, prop_schema) in &obj.properties {
        let type_ref = lower_type_ref(prop_schema)?;
        // Map string+binary to Bytes
        let type_ref = match &type_ref {
            TypeRef::Primitive(PrimitiveType::String) => {
                if let ObjectOrReference::Object(s) = prop_schema {
                    if s.format.as_deref() == Some("binary") {
                        TypeRef::Primitive(PrimitiveType::Bytes)
                    } else {
                        type_ref
                    }
                } else {
                    type_ref
                }
            }
            _ => type_ref,
        };
        fields.push(Field {
            name: prop_name.clone(),
            type_ref,
            required: obj.required.contains(prop_name),
            description: match prop_schema {
                ObjectOrReference::Object(s) => s.description.clone(),
                _ => None,
            },
        });
    }
    Ok(fields)
}

fn lower_responses(
    responses: &Option<std::collections::BTreeMap<String, ObjectOrReference<Response>>>,
    spec: &oas3::Spec,
) -> Result<Vec<OperationResponse>, Error> {
    let Some(responses) = responses else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();

    for (status_str, resp_ref) in responses {
        let resolved = match resp_ref {
            ObjectOrReference::Object(r) => r.clone(),
            ObjectOrReference::Ref { ref_path, .. } => {
                resp_ref.resolve(spec).map_err(|_| Error::UnresolvedRef {
                    ref_path: ref_path.clone(),
                })?
            }
        };

        let status_code = if status_str == "default" {
            ResponseStatusCode::Default
        } else {
            let code = status_str
                .parse::<u16>()
                .map_err(|_| Error::UnsupportedSchema {
                    context: format!("invalid status code: {status_str}"),
                })?;
            ResponseStatusCode::Code(code)
        };

        let body = RESPONSE_CONTENT_TYPE_PRIORITY
            .iter()
            .find_map(|(mime, ct)| resolved.content.get(*mime).map(|mt| (ct.clone(), mt)));

        let body = match body {
            Some((content_type, media_type)) => {
                let type_ref = match &content_type {
                    ContentType::TextPlain => TypeRef::Primitive(PrimitiveType::String),
                    ContentType::OctetStream => TypeRef::Primitive(PrimitiveType::Bytes),
                    ContentType::Json => match &media_type.schema {
                        Some(schema_or_ref) => lower_type_ref(schema_or_ref)?,
                        None => TypeRef::Primitive(PrimitiveType::String),
                    },
                    // These are request-only content types, won't appear in response priority list
                    ContentType::FormUrlEncoded | ContentType::MultipartFormData => unreachable!(),
                };
                Some(ResponseBody {
                    content_type,
                    type_ref,
                })
            }
            None => None,
        };

        result.push(OperationResponse {
            status_code,
            description: resolved.description.clone(),
            body,
        });
    }

    Ok(result)
}
