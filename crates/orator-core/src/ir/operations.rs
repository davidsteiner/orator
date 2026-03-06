use super::TypeRef;

/// A single API operation derived from a path+method in the OpenAPI spec.
#[derive(Debug, Clone, PartialEq)]
pub struct OperationIr {
    pub operation_id: String,
    pub description: Option<String>,
    pub method: HttpMethod,
    pub path: String,
    pub tag: Option<String>,
    pub parameters: Vec<OperationParam>,
    pub request_body: Option<RequestBodyIr>,
    pub responses: Vec<OperationResponse>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationParam {
    pub name: String,
    pub description: Option<String>,
    pub location: ParamLocation,
    pub type_ref: TypeRef,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamLocation {
    Path,
    Query,
    Header,
    Cookie,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestBodyIr {
    pub type_ref: TypeRef,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationResponse {
    pub status_code: ResponseStatusCode,
    pub description: Option<String>,
    pub body: Option<TypeRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseStatusCode {
    Code(u16),
    Default,
}
