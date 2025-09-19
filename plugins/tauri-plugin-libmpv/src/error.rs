use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error("Unsupported platform for mpv integration")]
    UnsupportedPlatform,
    #[error("Failed to get window handle")]
    WindowHandleError,
    #[error("A libmpv error occurred: {0}")]
    Mpv(String),
    #[error("mpv initialization failed: {0}")]
    Initialization(String),
    #[error("Failed to create mpv client: {0}")]
    ClientCreation(String),
    #[error("Command Error: {0}")]
    Command(String),
    #[error("Set Property Error: {0}")]
    SetProperty(String),
    #[error("Get Property Error: {0}")]
    GetProperty(String),
}

impl From<libmpv2::Error> for Error {
    fn from(e: libmpv2::Error) -> Self {
        Error::Mpv(e.to_string())
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
