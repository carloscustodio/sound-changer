// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use cpal::traits::{DeviceTrait, HostTrait};

#[derive(serde::Serialize, serde::Deserialize)]
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

#[tauri::command]
fn set_audio_devices( output: AudioDevice) -> Result<(), String> {
 
    #[cfg(target_os = "windows")]
    {
        // Safety: this call will attempt to switch the system's default output (sink)
        // device by its friendly name.
        default_device_sink::set_output_device(Some(output.name.clone()));
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("Changing default output device is only implemented on Windows".to_string());
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_audio_devices,
            set_audio_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
