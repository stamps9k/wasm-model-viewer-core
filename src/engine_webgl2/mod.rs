use wasm_bindgen::prelude::*;
use js_sys::Map;
use std::rc::Rc;
use std::cell::RefCell;

use crate::webgl2_frame::*;
use crate::webgl2_frame::animations::*;
use crate::logger::*;

#[wasm_bindgen]
pub struct EngineWebGl2
{
	#[allow(dead_code)] // read from JS via wasm-bindgen generated accessors
	frame: Rc<RefCell<WebGl2Frame>>,
} 

#[wasm_bindgen]
impl EngineWebGl2 
{
	#[wasm_bindgen(constructor)]
    pub fn new(resources: Map) -> Result<Self, JsValue> 
	{
		rust_log(&"Initialising webgl...", &"info_wasm_scene");
		let frame = Rc::new
		(
			RefCell::new
			(
				WebGl2Frame::new(resources)?
			)
		);
		initialize_animation(&frame.clone());
		rust_log(&"...webgl initialisation compplete.", &"info_wasm_scene");

		return Ok
		(
			Self
			{
				frame: frame
			}
		);
	}

	pub fn update_scene(&self, resources: Map) -> Result<(), JsValue>
	{
		rust_log(&"Refeshing scene with new data...", &"info_wasm_scene");
		let _ = self.frame.borrow_mut().update_scene(resources);
		rust_log(&"...scene refresh complete.", &"info_wasm_scene");
		return Ok(());
	}
}