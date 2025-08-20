// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(serde::Serialize)]
struct AudioDevice {
    name: String,
}

#[tauri::command]
fn get_audio_devices() -> (Vec<AudioDevice>, Vec<AudioDevice>) {
    let host = cpal::default_host();
    let mut input_devices = Vec::new();
    let mut output_devices = Vec::new();

    if let Ok(devices) = host.devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if device.default_input_config().is_ok() {
                    input_devices.push(AudioDevice { name: name.clone() });
                }
                if device.default_output_config().is_ok() {
                    output_devices.push(AudioDevice { name });
                }
            }
        }
    }
    (input_devices, output_devices)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_audio_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
