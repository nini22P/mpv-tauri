use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error(transparent)]
    Mpv(#[from] crate::libmpv::Error),
    #[error("Unsupported platform {0}")]
    UnsupportedPlatform(String),
    #[error("Not found window with label: '{0}'")]
    WindowNotFound(String),
    #[error("Failed to get window handle: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
    #[error("mpv instance not found: {0}")]
    InstanceNotFound(String),
    #[error("Command Error: {0}")]
    Command(String),
    #[error("Set Property Error: {0}")]
    SetProperty(String),
    #[error("Get Property Error: {0}")]
    GetProperty(String),
    #[error("Failed to destroy mpv instance: {0}")]
    Destroy(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
