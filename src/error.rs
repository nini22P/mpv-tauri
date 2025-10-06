use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error("Not found window with label: '{0}'")]
    WindowNotFound(String),
    #[error("Unsupported platform {0}")]
    UnsupportedPlatform(String),
    #[error("Failed to get window handle: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
    #[error("mpv process failed: {0}")]
    MpvProcessError(String),
    #[error("IPC communication error: {0}")]
    IpcError(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
