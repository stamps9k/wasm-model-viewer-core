mod controller;
mod utils;
mod logger;
mod engine_webgl2;
mod webgl2_frame;
mod webgl2_wavefront_object;
mod matrix_helper;

use crate::controller::*;
use crate::utils::*;
use crate::engine_webgl2::*;

use wasm_bindgen::prelude::*;
use web_sys::*;
use js_sys::Map;

#[wasm_bindgen]
pub fn initialize_web_gl(resources: Map) -> Result<EngineWebGl2, JsValue> 
{
	set_panic_hook();

	//Register all required event listeners for interactivity
	register_get_mouse_position();

	return EngineWebGl2::new(resources);	
}

#[wasm_bindgen]
pub fn update_scene(engine: EngineWebGl2, resources: Map) -> Result<EngineWebGl2, JsValue>
{
	let _ = engine.update_scene(resources);

	return Ok(engine);
}

#[wasm_bindgen]
pub fn enable_rotate_x()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_x = true;
}

#[wasm_bindgen]
pub fn disable_rotate_x()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_x = false;
}

#[wasm_bindgen]
pub fn enable_rotate_y()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_y = true;
}

#[wasm_bindgen]
pub fn disable_rotate_y()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_y = false;
}

#[wasm_bindgen]
pub fn enable_rotate_z()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_z = true;
}

#[wasm_bindgen]
pub fn disable_rotate_z()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.rotate_z = false;
}

#[wasm_bindgen]
pub fn enable_zoom_in()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.zoom_in = true;
}

#[wasm_bindgen]
pub fn disable_zoom_in()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.zoom_in = false;
}

#[wasm_bindgen]
pub fn enable_zoom_out()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.zoom_out = true;
}

#[wasm_bindgen]
pub fn disable_zoom_out()
{
	let controller_values = get_control_flags();
	let mut tmp = controller_values.lock().unwrap();
	tmp.zoom_out = false;
}