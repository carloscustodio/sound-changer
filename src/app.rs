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
    name: String,
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
    let input_devices = use_state(Vec::new);
    let output_devices = use_state(Vec::new);
    let last_staged = use_state(|| None::<String>);

    {
        use_effect_with((), move |_| {
            spawn_local(async move {
                let devices_val = unsafe { invoke("get_audio_devices", JsValue::NULL).await };
                match from_value::<(Vec<AudioDevice>, Vec<AudioDevice>)>(devices_val) {
                    Ok(devices) => {
                        input_devices.set(devices.0);
                        output_devices.set(devices.1);
                    }
                    Err(e) => {
                        // Handle error, e.g., log to console
                        web_sys::console::error_1(&e.into());
                    }
                }
            });
            || ()
        });
    }

    // Listen for tile:dropped and detect drops into the staging area
    {
        let last_staged = last_staged.clone();
        use_effect_with((), move |_| {
            let cb =
                Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |e: web_sys::Event| {
                    if e.type_() == "tile:dropped" {
                        let any: &js_sys::Object = e.unchecked_ref();
                        let detail = js_sys::Reflect::get(any, &JsValue::from_str("detail"))
                            .unwrap_or(JsValue::UNDEFINED);
                        if let Ok(d) = serde_wasm_bindgen::from_value::<DropDetail>(detail) {
                            if d.to_list_id.as_deref() == Some("staging-list") {
                                if let Some(id) = d.tile_id {
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
                }));
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("tile:dropped", cb.as_ref().unchecked_ref())
                .ok();
            cb.forget();
            || ()
        });
    }

    let input_list = input_devices
        .iter()
        .map(|d| {
            html! {
            <div class="device-row draggable-tile" data-tile-id={format!("input-{}", d.name)}>
                        <Tile title={Some(d.name.clone())} tilt={true}>
                            {"Input Device"}
                        </Tile>
                    </div>
                }
        })
        .collect::<Html>();

    let output_list = output_devices
        .iter()
        .map(|d| {
            html! {
            <div class="device-row draggable-tile" data-tile-id={format!("output-{}", d.name)}>
                        <Tile title={Some(d.name.clone())} tilt={true}>
                            {"Output Device"}
                        </Tile>
                    </div>
                }
        })
        .collect::<Html>();

    html! {
        <MetroProvider>
            <div class="devices-container">
                <div class="device-section">
                    <h2 class="section-title">{"Input Devices"}</h2>
                    <div class="device-list" id="input-list">
                        { input_list }
                    </div>
                </div>

                <div class="device-section">
                    <h2 class="section-title">{"Output Devices"}</h2>
                    <div class="device-list" id="output-list">
                        { output_list }
                    </div>
                </div>

                <div class="device-section">
                    <h2 class="section-title">{"Staging"}</h2>
                    <div class="device-list" id="staging-list">
                        { if let Some(id) = (*last_staged).clone() { html!{ <div class="staging-msg">{format!("Last staged: {}", id)}</div> } } else { Html::default() } }
                    </div>
                </div>
            </div>
        </MetroProvider>
    }
}
