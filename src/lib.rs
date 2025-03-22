mod components;
mod filesystem;
use yew::prelude::*;
use wasm_bindgen::prelude::*;
//use yew::prelude::*;

use crate::components::desktop::Desktop;
//use crate::filesystem::FileSystem;

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Starting KernelOS...");
    
    yew::Renderer::<Desktop>::new().render();
    
    Ok(())
} 