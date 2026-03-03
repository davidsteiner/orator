use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to resolve reference: {ref_path}")]
    UnresolvedRef { ref_path: String },

    #[error("unsupported schema: {context}")]
    UnsupportedSchema { context: String },
}
