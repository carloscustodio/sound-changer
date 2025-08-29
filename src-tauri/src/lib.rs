mod audio_manager;
mod error;

use audio_manager::{AudioDevice, AudioManager, DeviceType};
use error::AudioResult;
use std::sync::Arc;
use tauri::State;
use tracing::info;

// Application State
pub struct AppState {
    pub audio_manager: Arc<AudioManager>,
}

#[tauri::command]
async fn get_audio_devices(state: State<'_, AppState>) -> AudioResult<Vec<AudioDevice>> {
    info!("Fetching audio devices...");
    state.audio_manager.get_devices().await
}

#[tauri::command]
async fn set_default_device(
    device_id: String,
    device_type: String,
    state: State<'_, AppState>,
) -> AudioResult<()> {
    info!("Setting default device: {} ({})", device_id, device_type);

    let device_type = match device_type.as_str() {
        "Playback" => DeviceType::Playback,
        "Recording" => DeviceType::Recording,
        _ => {
            return Err(error::AudioError::ParseError(
                "Invalid device type".to_string(),
            ))
        }
    };

    state
        .audio_manager
        .set_default_device(&device_id, &device_type)
        .await
}

#[tauri::command]
async fn check_module_availability(state: State<'_, AppState>) -> AudioResult<bool> {
    info!("Checking AudioDeviceCmdlets module availability...");
    state.audio_manager.check_module_availability().await
}

#[tauri::command]
async fn install_audio_module(state: State<'_, AppState>) -> AudioResult<()> {
    info!("Installing AudioDeviceCmdlets module...");
    state.audio_manager.install_module().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create application state
    let audio_manager = Arc::new(AudioManager::new().expect("Failed to initialize AudioManager"));
    let app_state = AppState { audio_manager };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            set_default_device,
            check_module_availability,
            install_audio_module
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
