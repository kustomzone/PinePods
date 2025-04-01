const COMMANDS: &[&str] = &["ping", "register_media_session", "update_playback_state"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
