use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct AudioDevice {
    id: String,
    name: String,
    device_type: String,
    state: String,
    is_default: bool,
    is_communication_default: bool,
    last_seen: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DevicePriority {
    device_id: String,
    device_name: String,
    device_type: String,
    priority: usize, // 0 = highest priority
}

#[derive(Clone, Debug)]
struct AppError {
    message: String,
    timestamp: f64,
}

#[derive(Clone, Debug)]
struct AppState {
    devices: Vec<AudioDevice>,
    last_refresh: Option<f64>,
    error: Option<AppError>,
    module_available: Option<bool>,
    playback_priorities: Vec<DevicePriority>,
    recording_priorities: Vec<DevicePriority>,
    auto_switch_enabled: bool,
    installing_module: bool,
}

use subwayui::MetroProvider;

// Auto-switching logic
async fn auto_switch_device(priorities: &[DevicePriority], available_devices: &[AudioDevice], device_type: &str) -> Option<String> {
    // Find the highest priority device that's available
    for priority in priorities {
        if let Some(device) = available_devices.iter().find(|d| d.id == priority.device_id && d.state == "Active") {
            // Check if this device is already the default
            if !device.is_default {
                // Try to set this as the default device
                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "args": {
                        "device_id": device.id,
                        "device_type": device_type
                    }
                })).unwrap();
                
                let result = invoke("set_default_device", args).await;
                // Backend returns Result<(), _>; parse unit to confirm success
                if from_value::<()>(result).is_ok() {
                    return Some(device.name.clone());
                }
            } else {
                // Already the default, no need to switch
                return None;
            }
        }
    }
    None
}

// Storage functions for priority persistence
fn save_priorities_to_storage(playback: &[DevicePriority], recording: &[DevicePriority]) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(playback_json) = serde_json::to_string(playback) {
                let _ = storage.set_item("playback_priorities", &playback_json);
            }
            if let Ok(recording_json) = serde_json::to_string(recording) {
                let _ = storage.set_item("recording_priorities", &recording_json);
            }
        }
    }
}

fn load_priorities_from_storage() -> (Vec<DevicePriority>, Vec<DevicePriority>) {
    let mut playback_priorities = Vec::new();
    let mut recording_priorities = Vec::new();
    
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(playback_json)) = storage.get_item("playback_priorities") {
                if let Ok(priorities) = serde_json::from_str::<Vec<DevicePriority>>(&playback_json) {
                    playback_priorities = priorities;
                }
            }
            if let Ok(Some(recording_json)) = storage.get_item("recording_priorities") {
                if let Ok(priorities) = serde_json::from_str::<Vec<DevicePriority>>(&recording_json) {
                    recording_priorities = priorities;
                }
            }
        }
    }
    
    (playback_priorities, recording_priorities)
}

// Helper function to render priority chain like a flow diagram with 5 fixed positions
fn render_priority_chain(priorities: &[DevicePriority], list_type: String, _app_state: &UseStateHandle<AppState>, available_devices: &[AudioDevice]) -> Html {
    // Create 5 fixed priority positions (0 = highest priority, 4 = lowest priority)
    let priority_slots = (0..5).map(|slot_index| {
        // Find device assigned to this priority slot
        let assigned_device = priorities.iter().find(|p| p.priority == slot_index);
        
        let priority_description = match slot_index {
            0 => "Highest Priority",
            1 => "High Priority", 
            2 => "Medium Priority",
            3 => "Low Priority",
            4 => "Lowest Priority",
            _ => "Priority"
        };
        
        match assigned_device {
            Some(priority) => {
                // Slot has a device assigned
                let is_available = available_devices.iter().any(|d| d.id == priority.device_id && d.state == "Active");
                let is_current = available_devices.iter().any(|d| d.id == priority.device_id && d.is_default);
                
                let mut box_class = "priority-box filled".to_string();
                if is_current {
                    box_class.push_str(" current-device");
                } else if is_available {
                    box_class.push_str(" available");  
                } else {
                    box_class.push_str(" unavailable");
                }
                
                let status_text = if is_current {
                    "🔊 ACTIVE"
                } else if is_available {
                    "✓ Available" 
                } else {
                    "✗ Disconnected"
                };
                
                html! {
                    <div key={format!("slot-{}", slot_index)} class="priority-slot">
                        <div 
                            class={box_class}
                            data-priority-slot={slot_index.to_string()}
                            data-chain-type={list_type.clone()}
                        >
                            <div class="priority-header">
                                <div class="priority-number">{slot_index + 1}</div>
                                <div class="priority-label">{priority_description}</div>
                            </div>
                            <div class="device-info">
                                <div class="device-name">{&priority.device_name}</div>
                                <div class="device-status">{status_text}</div>
                            </div>
                            <div class="priority-actions">
                                <button class="priority-btn remove" title="Remove">{"✕"}</button>
                            </div>
                        </div>
                        if slot_index < 4 {
                            <div class="priority-arrow">{"→"}</div>
                        }
                    </div>
                }
            }
            None => {
                // Empty slot ready for device drop
                html! {
                    <div key={format!("slot-{}", slot_index)} class="priority-slot">
                        <div 
                            class="priority-box empty"
                            data-priority-slot={slot_index.to_string()}
                            data-chain-type={list_type.clone()}
                        >
                            <div class="priority-header">
                                <div class="priority-number">{slot_index + 1}</div>
                                <div class="priority-label">{priority_description}</div>
                            </div>
                            <div class="empty-slot-content">
                                <div class="drop-hint">{"Drop device here"}</div>
                                <div class="drop-icon">{"📥"}</div>
                            </div>
                        </div>
                        if slot_index < 4 {
                            <div class="priority-arrow">{"→"}</div>
                        }
                    </div>
                }
            }
        }
    }).collect::<Html>();
    
    html! {
        <div class="priority-chain-container">
            <div class="priority-flow horizontal">
                <div class="priority-direction-label">{"Highest Priority"}</div>
                {priority_slots}
                <div class="priority-direction-label">{"Lowest Priority"}</div>
            </div>
            <div class="priority-instructions">
                {"Drag audio devices into the boxes above to set their priority order. Priority 1 = Highest, Priority 5 = Lowest"}
            </div>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let app_state = use_state(|| {
        let (playback_priorities, recording_priorities) = load_priorities_from_storage();
        AppState {
            devices: Vec::new(),
            last_refresh: None,
            error: None,
            module_available: None,
            playback_priorities,
            recording_priorities,
            auto_switch_enabled: true,
            installing_module: false,
        }
    });

    // Load devices on startup
    {
        let app_state = app_state.clone();
        use_effect_with((), move |_| {
            let app_state = app_state.clone();
            spawn_local(async move {
                let devices_val = invoke("get_audio_devices", JsValue::NULL).await;
                if let Ok(devices) = from_value::<Vec<AudioDevice>>(devices_val) {
                    let mut state = (*app_state).clone();
                    state.devices = devices;
                    state.last_refresh = Some(js_sys::Date::now());
                    app_state.set(state);
                }
            });
            || ()
        });
    }

    // Check if AudioDeviceCmdlets module is available on startup
    {
        let app_state = app_state.clone();
        use_effect_with((), move |_| {
            let app_state = app_state.clone();
            spawn_local(async move {
                let availability = invoke("check_module_availability", JsValue::NULL).await;
                if let Ok(is_available) = from_value::<bool>(availability) {
                    let mut state = (*app_state).clone();
                    state.module_available = Some(is_available);
                    app_state.set(state);
                }
            });
            || ()
        });
    }

    // Auto-switching effect - triggers when devices change and auto-switch is enabled
    {
        let app_state = app_state.clone();
        let devices = app_state.devices.clone();
        let auto_switch_enabled = app_state.auto_switch_enabled;
        let playback_priorities = app_state.playback_priorities.clone();
        let recording_priorities = app_state.recording_priorities.clone();
        
        use_effect_with((devices.len(), auto_switch_enabled), move |_| {
            if auto_switch_enabled && !devices.is_empty() {
                let app_state = app_state.clone();
                let devices = devices.clone();
                let playback_priorities = playback_priorities.clone();
                let recording_priorities = recording_priorities.clone();
                
                spawn_local(async move {
                    let playback_devices: Vec<AudioDevice> = devices
                        .iter()
                        .filter(|d| d.device_type == "Playback")
                        .cloned()
                        .collect();
                    
                    let recording_devices: Vec<AudioDevice> = devices
                        .iter()
                        .filter(|d| d.device_type == "Recording")
                        .cloned()
                        .collect();
                    
                    // Try auto-switching for playback devices
                    if let Some(switched_device) = auto_switch_device(&playback_priorities, &playback_devices, "Playback").await {
                        web_sys::console::log_1(&format!("Auto-switched to playback device: {}", switched_device).into());
                        
                        // Refresh devices to show the change
                        let devices_val = invoke("get_audio_devices", JsValue::NULL).await;
                        if let Ok(devices) = from_value::<Vec<AudioDevice>>(devices_val) {
                            let mut state = (*app_state).clone();
                            state.devices = devices;
                            state.last_refresh = Some(js_sys::Date::now());
                            app_state.set(state);
                        }
                    }
                    
                    // Try auto-switching for recording devices  
                    if let Some(switched_device) = auto_switch_device(&recording_priorities, &recording_devices, "Recording").await {
                        web_sys::console::log_1(&format!("Auto-switched to recording device: {}", switched_device).into());
                        
                        // Refresh devices to show the change
                        let devices_val = invoke("get_audio_devices", JsValue::NULL).await;
                        if let Ok(devices) = from_value::<Vec<AudioDevice>>(devices_val) {
                            let mut state = (*app_state).clone();
                            state.devices = devices;
                            state.last_refresh = Some(js_sys::Date::now());
                            app_state.set(state);
                        }
                    }
                });
            }
            || ()
        });
    }

    // Auto-switch toggle callback
    let toggle_auto_switch = {
        let app_state = app_state.clone();
        Callback::from(move |_| {
            let mut state = (*app_state).clone();
            state.auto_switch_enabled = !state.auto_switch_enabled;
            
            // Save the setting to localStorage
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("auto_switch_enabled", &state.auto_switch_enabled.to_string());
                }
            }
            
            app_state.set(state);
        })
    };

    // Install AudioDeviceCmdlets on demand
    let install_module = {
        let app_state = app_state.clone();
        Callback::from(move |_| {
            let app_state = app_state.clone();
            spawn_local(async move {
                // mark installing
                {
                    let mut s = (*app_state).clone();
                    s.installing_module = true;
                    app_state.set(s);
                }

                let install_res = invoke("install_audio_module", JsValue::NULL).await;
                let _ok = from_value::<()>(install_res).is_ok();

                // re-check availability
                let availability = invoke("check_module_availability", JsValue::NULL).await;
                let is_available = from_value::<bool>(availability).unwrap_or(false);

                let mut s = (*app_state).clone();
                s.module_available = Some(is_available);
                s.installing_module = false;
                app_state.set(s);
            });
        })
    };

    // Native HTML5 DnD handlers are removed to avoid conflicts with InteractJS

    // Filter devices by type
    let recording_devices: Vec<AudioDevice> = app_state
        .devices
        .iter()
        .filter(|d| d.device_type == "Recording")
        .cloned()
        .collect();

    let playback_devices: Vec<AudioDevice> = app_state
        .devices
        .iter()
        .filter(|d| d.device_type == "Playback")
        .cloned()
        .collect();

    html! {
        <MetroProvider>
            <div class="app-container">
                <div class="status-bar">
                    <div class="refresh-time">
                        {format!("Devices loaded: {}", app_state.devices.len())}
                    </div>
                    {
                        match app_state.module_available {
                            Some(true) => html! { <div class="module-status ok">{"AudioDeviceCmdlets: Available"}</div> },
                            Some(false) => html! {
                                <div class="module-status warn">
                                    {"AudioDeviceCmdlets: Missing "}
                                    <button 
                                        type="button" 
                                        class="install-module-btn"
                                        onclick={install_module.clone()}
                                        disabled={app_state.installing_module}
                                    >
                                        { if app_state.installing_module { "Installing..." } else { "Install now" } }
                                    </button>
                                </div>
                            },
                            None => html! { <div class="module-status">{"Checking module..."}</div> },
                        }
                    }
                </div>
                
                <div class="devices-container">
                    <div class="device-section">
                        <h2 class="section-title">{format!("Available Playback Devices ({})", playback_devices.len())}</h2>
                        <div class="tiles-grid">
                            {
                                playback_devices.iter().map(|d| {
                                    let wrapper_class = if d.is_default { "default-device" } else { "" };
                                    let device_id = d.id.clone();
                                    let device_name = d.name.clone();
                                    let device_type = d.device_type.clone();

                                    html! {
                                        <div class={wrapper_class} key={d.id.clone()}>
                                            <div
                                                class="subway-tile"
                                                data-device-id={device_id}
                                                data-device-name={device_name}
                                                data-device-type={device_type}
                                                style="cursor: grab; user-select: none;"
                                            >
                                                <div class="subway-tile-title">{"Playback"}</div>
                                                <div class="subway-tile-body">
                                                    <div class="tile-title">{d.name.clone()}</div>
                                                    <div class="tile-status">{ if d.is_default { "Default" } else { "Available" } }</div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    </div>

                    <div class="device-section">
                        <h2 class="section-title">{"Playback Priority Chain (Highest → Lowest)"}</h2>
                        <div class="priority-controls">
                            <label class="auto-switch-toggle">
                                <button 
                                    type="button" 
                                    class="auto-switch-button"
                                    onclick={toggle_auto_switch}
                                >
                                    {"🔄 Auto-switch to highest priority available device"}
                                </button>
                            </label>
                        </div>
                        <div 
                            class="priority-chain"
                            data-chain-type="playback"
                        >
                            {render_priority_chain(&app_state.playback_priorities, "playback".to_string(), &app_state, &playback_devices)}
                        </div>
                    </div>

                    <div class="device-section">
                        <h2 class="section-title">{format!("Available Recording Devices ({})", recording_devices.len())}</h2>
                        <div class="tiles-grid">
                            {
                                recording_devices.iter().map(|d| {
                                    let wrapper_class = if d.is_default { "default-device" } else { "" };
                                    let device_id = d.id.clone();
                                    let device_name = d.name.clone();
                                    let device_type = d.device_type.clone();

                                    html! {
                                        <div class={wrapper_class} key={d.id.clone()}>
                                            <div
                                                class="subway-tile"
                                                data-device-id={device_id}
                                                data-device-name={device_name}
                                                data-device-type={device_type}
                                                style="cursor: grab; user-select: none;"
                                            >
                                                <div class="subway-tile-title">{"Recording"}</div>
                                                <div class="subway-tile-body">
                                                    <div class="tile-title">{d.name.clone()}</div>
                                                    <div class="tile-status">{ if d.is_default { "Default" } else { "Available" } }</div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    </div>

                    <div class="device-section">
                        <h2 class="section-title">{"Recording Priority Chain (Highest → Lowest)"}</h2>
                        <div 
                            class="priority-chain"
                            data-chain-type="recording"
                        >
                            {render_priority_chain(&app_state.recording_priorities, "recording".to_string(), &app_state, &recording_devices)}
                        </div>
                    </div>
                </div>
            </div>
        </MetroProvider>
    }
}
