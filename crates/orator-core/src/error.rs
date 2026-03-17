use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to resolve reference: {ref_path}")]
    UnresolvedRef { ref_path: String },

    #[error("unsupported schema: {context}")]
    UnsupportedSchema { context: String },

    #[error("missing operationId for {method} {path}")]
    MissingOperationId { method: String, path: String },

    #[error("unsupported media type for request body in {operation_id}: found {media_types:?}")]
    UnsupportedRequestBodyMediaType {
        operation_id: String,
        media_types: Vec<String>,
    },
}
