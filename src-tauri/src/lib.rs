mod audio_manager;
mod error;

use audio_manager::{AudioDevice, AudioManager, DeviceType};
use error::AudioResult;
use serde::Deserialize;
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

#[derive(Deserialize)]
struct SetDefaultArgs {
    #[serde(alias = "deviceId")]
    device_id: String,
    #[serde(alias = "deviceType")]
    device_type: String,
}

#[tauri::command]
async fn set_default_device(args: SetDefaultArgs, state: State<'_, AppState>) -> AudioResult<()> {
    let SetDefaultArgs {
        device_id,
        device_type,
    } = args;
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

#[derive(Deserialize)]
struct AddToSlotArgs {
    #[serde(alias = "deviceId")]
    device_id: String,
    #[serde(alias = "deviceName")]
    device_name: String,
    #[serde(alias = "deviceType")]
    device_type: String,
    #[serde(alias = "priorityType")]
    priority_type: String,
    #[serde(alias = "prioritySlot")]
    priority_slot: usize,
}

#[tauri::command]
async fn add_device_to_priority_slot(
    args: AddToSlotArgs,
    state: State<'_, AppState>,
) -> AudioResult<()> {
    let AddToSlotArgs {
        device_id,
        device_name,
        device_type,
        priority_type,
        priority_slot,
    } = args;
    info!(
        "Adding device {} to priority slot {} for {} devices",
        device_name, priority_slot, priority_type
    );

    // For now, just log the action since we're focusing on the UI
    // In a full implementation, this would save to a priority database or config file
    info!(
        "Device '{}' ({}) assigned to priority slot {} in {} chain",
        device_name, device_id, priority_slot, priority_type
    );

    Ok(())
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
            add_device_to_priority_slot,
            check_module_availability,
            install_audio_module
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
