use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
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
}

#[derive(Deserialize, Debug)]
struct DropDetail {
    #[serde(rename = "tileId")]
    tile_id: Option<String>,
    #[serde(rename = "toListId")]
    to_list_id: Option<String>,
}

use subwayui::{components::Tile, MetroProvider};

#[function_component(App)]
pub fn app() -> Html {
    let app_state = use_state(|| AppState {
        devices: Vec::new(),
        last_refresh: None,
        error: None,
        module_available: None,
    });
    let last_staged = use_state(|| None::<String>);
    let is_loading = use_state(|| false);

    // Check module availability on startup
    {
        let app_state = app_state.clone();
        use_effect_with((), move |_| {
            let app_state = app_state.clone();
            spawn_local(async move {
                let result = invoke("check_module_availability", JsValue::NULL).await;
                if let Ok(available) = from_value::<bool>(result) {
                    let mut state = (*app_state).clone();
                    state.module_available = Some(available);
                    app_state.set(state);

                    if !available {
                        web_sys::console::warn_1(&JsValue::from_str(
                            "⚠️ AudioDeviceCmdlets module not available. Some features may not work."
                        ));
                    }
                } else {
                    web_sys::console::error_1(&JsValue::from_str(
                        "Failed to check module availability",
                    ));
                }
            });
            || ()
        });
    }

    // Enhanced device loading with error handling
    {
        let app_state = app_state.clone();
        let is_loading = is_loading.clone();

        use_effect_with((), move |_| {
            let app_state = app_state.clone();
            let is_loading = is_loading.clone();

            spawn_local(async move {
                is_loading.set(true);

                let start_time = js_sys::Date::now();

                let devices_val = invoke("get_audio_devices", JsValue::NULL).await;
                match from_value::<Vec<AudioDevice>>(devices_val) {
                    Ok(devices) => {
                        let mut state = (*app_state).clone();
                        state.devices = devices;
                        state.last_refresh = Some(js_sys::Date::now());
                        state.error = None;
                        app_state.set(state);

                        let load_time = js_sys::Date::now() - start_time;
                        web_sys::console::log_1(&JsValue::from_str(&format!(
                            "🔊 Loaded {} devices in {:.0}ms",
                            (*app_state).devices.len(),
                            load_time
                        )));
                    }
                    Err(e) => {
                        let mut state = (*app_state).clone();
                        state.error = Some(AppError {
                            message: format!("Failed to load or parse devices: {:?}", e),
                            timestamp: js_sys::Date::now(),
                        });
                        app_state.set(state);

                        web_sys::console::error_1(&JsValue::from_str(&format!(
                            "❌ Failed to load audio devices: {:?}",
                            e
                        )));
                    }
                }

                is_loading.set(false);
            });
            || ()
        });
    }

    // Listen for tile:dropped and detect drops into the staging area
    {
        let last_staged = last_staged.clone();
        let app_state = app_state.clone();

        use_effect_with((), move |_| {
            let cb = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
                move |e: web_sys::Event| {
                    if e.type_() == "tile:dropped" {
                        let any: &js_sys::Object = e.unchecked_ref();
                        let detail = js_sys::Reflect::get(any, &JsValue::from_str("detail"))
                            .unwrap_or(JsValue::UNDEFINED);
                        if let Ok(d) = serde_wasm_bindgen::from_value::<DropDetail>(detail) {
                            if d.to_list_id.as_deref() == Some("staging-list") {
                                if let Some(id) = d.tile_id {
                                    // Extract device ID from tile ID (format: "input-DeviceID" or "output-DeviceID")
                                    let device_id = if let Some(id_part) = id.strip_prefix("input-")
                                    {
                                        id_part.to_string()
                                    } else if let Some(id_part) = id.strip_prefix("output-") {
                                        id_part.to_string()
                                    } else {
                                        id.clone()
                                    };

                                    web_sys::console::log_1(&JsValue::from_str(&format!(
                                        "Extracted device ID: '{}'",
                                        device_id
                                    )));

                                    // Call set_default_device Tauri command
                                    let id_clone = id.clone(); // Clone before moving into async
                                    let app_state = app_state.clone();

                                    spawn_local(async move {
                                        // Determine device type from tile ID prefix
                                        let device_type = if id_clone.starts_with("input-") {
                                            "Recording"
                                        } else {
                                            "Playback"
                                        };

                                        // Create args manually
                                        let args = js_sys::Object::new();
                                        js_sys::Reflect::set(
                                            &args,
                                            &JsValue::from_str("device_id"),
                                            &JsValue::from_str(&device_id),
                                        )
                                        .ok();
                                        js_sys::Reflect::set(
                                            &args,
                                            &JsValue::from_str("device_type"),
                                            &JsValue::from_str(device_type),
                                        )
                                        .ok();

                                        web_sys::console::log_1(&JsValue::from_str(&format!(
                                            "Calling set_default_device with device_id: '{}', type: '{}'",
                                            device_id, device_type
                                        )));

                                        let start_time = js_sys::Date::now();

                                        let result =
                                            invoke("set_default_device", args.into()).await;
                                        let switch_time = js_sys::Date::now() - start_time;

                                        // Try to parse as string (error case)
                                        if let Ok(error_msg) =
                                            serde_wasm_bindgen::from_value::<String>(result.clone())
                                        {
                                            if !error_msg.is_empty() && error_msg != "null" {
                                                let mut state = (*app_state).clone();
                                                state.error = Some(AppError {
                                                    message: format!(
                                                        "Backend error: {}",
                                                        error_msg
                                                    ),
                                                    timestamp: js_sys::Date::now(),
                                                });
                                                app_state.set(state);

                                                web_sys::console::error_1(&JsValue::from_str(
                                                    &format!("❌ Backend error: {}", error_msg),
                                                ));
                                            } else {
                                                // Success case
                                                web_sys::console::log_1(&JsValue::from_str(&format!(
                                                    "✅ Successfully set default {} device to: {} ({:.0}ms)",
                                                    device_type.to_lowercase(),
                                                    device_id,
                                                    switch_time
                                                )));
                                            }
                                        } else {
                                            // Assume success if not an error string
                                            web_sys::console::log_1(&JsValue::from_str(&format!(
                                                "✅ Successfully set default {} device to: {} ({:.0}ms)",
                                                device_type.to_lowercase(),
                                                device_id,
                                                switch_time
                                            )));
                                        }
                                    });

                                    web_sys::console::log_1(&JsValue::from_str(&format!(
                                        "Dropped into staging: {}",
                                        id
                                    )));
                                    last_staged.set(Some(id));
                                } else {
                                    last_staged.set(Some("(unknown)".to_string()));
                                }
                            }
                        }
                    }
                },
            ));
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("tile:dropped", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
            || ()
        });
    }

    // Filter devices by type
    let input_devices: Vec<AudioDevice> = app_state
        .devices
        .iter()
        .filter(|d| d.device_type == "Recording")
        .cloned()
        .collect();

    let output_devices: Vec<AudioDevice> = app_state
        .devices
        .iter()
        .filter(|d| d.device_type == "Playback")
        .cloned()
        .collect();

    let input_list = input_devices
        .iter()
        .map(|d| {
            let mut tile_class = "device-row draggable-tile".to_string();
            if d.is_default {
                tile_class.push_str(" default-device");
            }
            if d.is_communication_default {
                tile_class.push_str(" communication-device");
            }

            let device_label = format!(
                "Input Device{}{}",
                if d.is_default { " (Default)" } else { "" },
                if d.is_communication_default {
                    " (Comm)"
                } else {
                    ""
                }
            );

            html! {
                <div class={tile_class} data-tile-id={format!("input-{}", d.id)}>
                    <Tile title={Some(d.name.clone())} tilt={true}>
                        {device_label}
                    </Tile>
                </div>
            }
        })
        .collect::<Html>();

    let output_list = output_devices
        .iter()
        .map(|d| {
            let mut tile_class = "device-row draggable-tile".to_string();
            if d.is_default {
                tile_class.push_str(" default-device");
            }
            if d.is_communication_default {
                tile_class.push_str(" communication-device");
            }

            let device_label = format!(
                "Output Device{}{}",
                if d.is_default { " (Default)" } else { "" },
                if d.is_communication_default {
                    " (Comm)"
                } else {
                    ""
                }
            );

            html! {
                <div class={tile_class} data-tile-id={format!("output-{}", d.id)}>
                    <Tile title={Some(d.name.clone())} tilt={true}>
                        {device_label}
                    </Tile>
                </div>
            }
        })
        .collect::<Html>();

    // Error notification component
    let error_display = if let Some(error) = &app_state.error {
        html! {
            <div class="error-notification">
                <div class="error-content">
                    <span class="error-icon">{"⚠️"}</span>
                    <span class="error-message">{&error.message}</span>
                    <button class="error-dismiss" onclick={
                        let app_state = app_state.clone();
                        Callback::from(move |_| {
                            let mut state = (*app_state).clone();
                            state.error = None;
                            app_state.set(state);
                        })
                    }>{"×"}</button>
                </div>
            </div>
        }
    } else {
        Html::default()
    };

    // Module status display
    let module_status = match app_state.module_available {
        Some(true) => {
            html! { <div class="status-ok">{"✅ AudioDeviceCmdlets module available"}</div> }
        }
        Some(false) => {
            html! { <div class="status-warning">{"⚠️ AudioDeviceCmdlets module not available"}</div> }
        }
        None => html! { <div class="status-loading">{"🔍 Checking module availability..."}</div> },
    };

    html! {
        <MetroProvider>
            <div class="app-container">
                {error_display}

                <div class="status-bar">
                    {module_status}
                    {
                        if *is_loading {
                            html! { <div class="loading-indicator">{"🔄 Loading devices..."}</div> }
                        } else if let Some(refresh_time) = app_state.last_refresh {
                            let date = js_sys::Date::new(&JsValue::from_f64(refresh_time));
                            html! { <div class="refresh-time">{format!("Last refresh: {}", date.to_locale_time_string("en-US"))}</div> }
                        } else {
                            Html::default()
                        }
                    }
                </div>

                <div class="devices-container">
                    <div class="device-section">
                        <h2 class="section-title">{format!("Input Devices ({})", input_devices.len())}</h2>
                        <div class="device-list" id="input-list">
                            { input_list }
                        </div>
                    </div>

                    <div class="device-section">
                        <h2 class="section-title">{format!("Output Devices ({})", output_devices.len())}</h2>
                        <div class="device-list" id="output-list">
                            { output_list }
                        </div>
                    </div>

                    <div class="device-section">
                        <h2 class="section-title">{"Staging"}</h2>
                        <div class="device-list" id="staging-list">
                            {
                                if let Some(id) = (*last_staged).clone() {
                                    html! { <div class="staging-msg">{format!("Last staged: {}", id)}</div> }
                                } else {
                                    html! { <div class="staging-msg staging-empty">{"Drop devices here to set as default"}</div> }
                                }
                            }
                        </div>
                    </div>
                </div>
            </div>
        </MetroProvider>
    }
}
