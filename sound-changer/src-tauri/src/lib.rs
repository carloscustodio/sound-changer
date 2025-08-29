mod audio_manager;
mod error;

pub fn initialize_audio_manager() {
    audio_manager::setup();
}

#[tauri::command]
pub fn change_audio_setting(setting: String) -> Result<(), error::AudioError> {
    audio_manager::change_setting(setting)
}