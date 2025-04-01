use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::MobileappmediaExt;
use crate::Result;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.mobileappmedia().ping(payload)
}

#[command]
pub(crate) async fn register_media_session<R: Runtime>(
    app: AppHandle<R>,
    title: String,
    artist: String,
    artwork_url: String,
    duration: f64,
) -> Result<()> {
    // Note the return type is Result<()>
    app.mobileappmedia()
        .register_media_session(title, artist, artwork_url, duration)
}

#[command]
pub(crate) async fn update_playback_state<R: Runtime>(
    app: AppHandle<R>,
    is_playing: bool,
    position: f64,
) -> Result<()> {
    // Note the return type is Result<()>
    app.mobileappmedia()
        .update_playback_state(is_playing, position)
}
