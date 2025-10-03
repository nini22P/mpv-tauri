#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create mpv handle")]
    Create,
    #[error("Failed to create mpv client handle")]
    ClientCreation,
    #[error("Failed to initialize mpv core: {0}")]
    Initialize(String),
    #[error("Failed to set option '{key}': {code}")]
    SetOption { key: String, code: String },
    #[error("Failed to get property '{key}': {code}")]
    GetProperty { key: String, code: String },
    #[error("Error processing event (id: {event_id}): {code}")]
    Event { code: String, event_id: String },
    #[error("Failed to create render context: {0}")]
    RenderContextCreation(String),
    #[error("Failed to render frame: {0}")]
    Render(String),
    #[error("Failed to execute command '{name}': {code}")]
    Command { name: String, code: String },
    #[error("Failed to set property '{key}': {code}")]
    SetProperty { key: String, code: String },
    #[error("Failed to observe property '{name}': {code}")]
    PropertyObserve { name: String, code: String },
    #[error("Failed to load config file at '{path}': {code}")]
    LoadConfig { path: String, code: String },
    #[error("Invalid C-style string provided")]
    InvalidCString(#[from] std::ffi::NulError),
    #[error("Failed to convert node: {0}")]
    NodeConversion(String),
    #[error("Failed to convert property: {0}")]
    PropertyConversion(String),
}
