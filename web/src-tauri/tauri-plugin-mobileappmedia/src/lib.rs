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
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Mobileappmedia;
#[cfg(mobile)]
use mobile::Mobileappmedia;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mobileappmedia APIs.
pub trait MobileappmediaExt<R: Runtime> {
    fn mobileappmedia(&self) -> &Mobileappmedia<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MobileappmediaExt<R> for T {
    fn mobileappmedia(&self) -> &Mobileappmedia<R> {
        self.state::<Mobileappmedia<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mobileappmedia")
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::register_media_session,
            commands::update_playback_state
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let mobileappmedia = mobile::init(app, api)?;
            #[cfg(desktop)]
            let mobileappmedia = desktop::init(app, api)?;
            app.manage(mobileappmedia);
            Ok(())
        })
        .build()
}
