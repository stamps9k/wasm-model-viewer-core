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
		rust_info(&"Initialising webgl...");
		let frame = Rc::new
		(
			RefCell::new
			(
				WebGl2Frame::new(resources)?
			)
		);
		initialize_animation(&frame.clone());
		rust_info(&"...webgl initialisation complete.");

		return Ok
		(
			Self
			{
				frame: frame
			}
		);
	}
}