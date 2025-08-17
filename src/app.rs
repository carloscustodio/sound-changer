use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::Element;
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

use subwayui::{components::Tile, MetroProvider};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <MetroProvider>
            <Tile title={Some("Metro Welcome".to_string())} tilt={true} tilt_max={18.0}>
                { "This is a subwayui tile with tilt enabled." }
            </Tile>
            <Tile title={Some("Static SubwayUI Tile".to_string())} tilt={false}>
                { "This is a static subwayui tile without tilt." }
            </Tile>
            <Tile title={Some("Custom Tilt".to_string())} tilt={true} tilt_max={10.0}>
                { "Another subwayui tile with custom tilt max." }
            </Tile>
        </MetroProvider>
    }
}
