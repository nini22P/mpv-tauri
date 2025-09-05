use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod events;
mod ipc;
mod models;
mod process;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Mpv;
#[cfg(mobile)]
use mobile::Mpv;

pub trait MpvExt<R: Runtime> {
    fn mpv(&self) -> &Mpv<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MpvExt<R> for T {
    fn mpv(&self) -> &Mpv<R> {
        self.state::<Mpv<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mpv")
        .invoke_handler(tauri::generate_handler![
            commands::initialize_mpv,
            commands::destroy_mpv,
            commands::send_mpv_command,
            commands::set_video_margin_ratio,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let mpv = mobile::init(app, api)?;
            #[cfg(desktop)]
            let mpv = desktop::init(app, api)?;
            app.manage(mpv);
            Ok(())
        })
        .build()
}
