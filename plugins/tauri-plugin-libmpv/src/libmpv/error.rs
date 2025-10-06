pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create mpv handle")]
    Create,
    #[error("Failed to create mpv client handle")]
    ClientCreation,
    #[error("Failed to initialize mpv core: {0}")]
    Initialize(String),
    #[error("Failed to set option '{name}': {code}")]
    SetOption { name: String, code: String },
    #[error("Failed to execute command '{name}': {code}")]
    Command { name: String, code: String },
    #[error("Failed to set property '{name}': {code}")]
    SetProperty { name: String, code: String },
    #[error("Failed to get property '{name}': {code}")]
    GetProperty { name: String, code: String },
    #[error("Error processing event (id: {event_id}): {code}")]
    Event { code: String, event_id: String },
    #[error("Failed to observe property '{name}': {code}")]
    PropertyObserve { name: String, code: String },
    #[error("Invalid C-style string provided")]
    InvalidCString(#[from] std::ffi::NulError),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Failed to convert node: {0}")]
    NodeConversion(String),
    #[error("Failed to convert property: {0}")]
    PropertyConversion(String),
}
