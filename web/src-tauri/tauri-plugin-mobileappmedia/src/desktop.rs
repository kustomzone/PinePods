use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;
use crate::Result;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Mobileappmedia<R>> {
    Ok(Mobileappmedia(app.clone()))
}

/// Access to the mobileappmedia APIs.
pub struct Mobileappmedia<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Mobileappmedia<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    pub fn register_media_session(
        &self,
        _title: String,
        _artist: String,
        _artwork_url: String,
        _duration: f64,
    ) -> crate::Result<()> {
        // No-op on desktop
        Ok(())
    }

    pub fn update_playback_state(&self, _is_playing: bool, _position: f64) -> crate::Result<()> {
        // No-op on desktop
        Ok(())
    }
}
