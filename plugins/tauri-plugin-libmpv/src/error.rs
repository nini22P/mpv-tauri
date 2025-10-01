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
    #[error("Not found window with label '{0}'")]
    WindowNotFound(String),
    #[error("Failed to get window handle: {0}")]
    WindowHandle(#[from] raw_window_handle::HandleError),
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

impl From<libmpv2::Error> for Error {
    fn from(e: libmpv2::Error) -> Self {
        let error_string = match e {
            libmpv2::Error::Raw(code) => {
                let error_name = mpv_error_code_to_name(code);
                format!("{} ({})", error_name, code)
            }
            _ => e.to_string(),
        };
        Error::Mpv(error_string)
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

pub fn mpv_error_code_to_name(code: i32) -> &'static str {
    use libmpv2::mpv_error;
    match code {
        mpv_error::Success => "Success",
        mpv_error::EventQueueFull => "EventQueueFull",
        mpv_error::NoMem => "NoMem",
        mpv_error::Uninitialized => "Uninitialized",
        mpv_error::InvalidParameter => "InvalidParameter",
        mpv_error::OptionNotFound => "OptionNotFound",
        mpv_error::OptionFormat => "OptionFormat",
        mpv_error::OptionError => "OptionError",
        mpv_error::PropertyNotFound => "PropertyNotFound",
        mpv_error::PropertyFormat => "PropertyFormat",
        mpv_error::PropertyUnavailable => "PropertyUnavailable",
        mpv_error::PropertyError => "PropertyError",
        mpv_error::Command => "Command",
        mpv_error::LoadingFailed => "LoadingFailed",
        mpv_error::AoInitFailed => "AoInitFailed",
        mpv_error::VoInitFailed => "VoInitFailed",
        mpv_error::NothingToPlay => "NothingToPlay",
        mpv_error::UnknownFormat => "UnknownFormat",
        mpv_error::Unsupported => "Unsupported",
        mpv_error::NotImplemented => "NotImplemented",
        mpv_error::Generic => "Generic",
        _ => "UnknownErrorCode",
    }
}
