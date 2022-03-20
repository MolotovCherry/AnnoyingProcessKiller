use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WMIError {
    #[error("Null pointer was sent as part of query result")]
    NullPointerResult,

    #[error("Failed to create IWbemContext")]
    IWbemContextCreateFailed,

    #[error("Query run failed")]
    QueryRunFailed,

    #[error("Unparsable Query")]
    WbemUnparsableQuery,

    #[error("Property is not a CIM_OBJECT")]
    NotCimObject
}
