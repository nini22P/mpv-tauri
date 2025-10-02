use serde::{ser::Serializer, Serialize};

use crate::libmpv;

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
    #[error("Not found window with label '{0}'")]
    WindowNotFound(String),
    #[error("Failed to get window handle: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
    #[error("mpv instance not found: {0}")]
    InstanceNotFound(String),
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
    #[error("Invalid format string provided: {0}")]
    Format(String),
}

impl From<libmpv::Error> for Error {
    fn from(e: libmpv::Error) -> Self {
        match e {
            libmpv::Error::Command { name, code } => {
                Error::Command(format!("Command '{}' failed: {}", name, code))
            }
            libmpv::Error::SetProperty { key, code } => {
                Error::SetProperty(format!("Property '{}' failed: {}", key, code))
            }
            _ => Error::Mpv(e.to_string()),
        }
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
