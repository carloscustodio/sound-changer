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

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
struct AudioDevice {
    name: String,
}

use subwayui::{components::Tile, MetroProvider};

#[function_component(App)]
pub fn app() -> Html {
    let input_devices = use_state(Vec::new);
    let output_devices = use_state(Vec::new);

    {
        let input_devices = input_devices.clone();
        let output_devices = output_devices.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let devices_val = invoke("get_audio_devices", JsValue::NULL).await;
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

    let input_list = input_devices
        .iter()
        .map(|d| html! { <Tile title={Some(d.name.clone())} tilt={true}> </Tile> })
        .collect::<Html>();
    let output_list = output_devices
        .iter()
        .map(|d| html! { <Tile title={Some(d.name.clone())} tilt={true}> </Tile> })
        .collect::<Html>();

    html! {
        <MetroProvider>
            <div>{ input_list }</div>
                <div>{ output_list }</div>

        </MetroProvider>
    }
}
