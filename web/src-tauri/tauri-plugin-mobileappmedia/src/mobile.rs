use serde::de::DeserializeOwned;
use serde_json::{from_str, json};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;
use crate::Result;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_mobileappmedia);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Mobileappmedia<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("com.plugin.mobileappmedia", "MediaSessionPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_mobileappmedia)?;
    Ok(Mobileappmedia(handle))
}

/// Access to the mobileappmedia APIs.
pub struct Mobileappmedia<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Mobileappmedia<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        self.0
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
    }

    pub fn register_media_session(
        &self,
        title: String,
        artist: String,
        artwork_url: String,
        duration: f64,
    ) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<serde_json::Value>(
                "registerMediaSession",
                serde_json::json!({
                    "title": title,
                    "artist": artist,
                    "artworkUrl": artwork_url,
                    "duration": duration
                }),
            )
            .map(|_| ())
            .map_err(Into::into)
    }

    pub fn update_playback_state(&self, is_playing: bool, position: f64) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<serde_json::Value>(
                "updatePlaybackState",
                serde_json::json!({
                    "playState": is_playing,
                    "position": position
                }),
            )
            .map(|_| ())
            .map_err(Into::into)
    }
}
