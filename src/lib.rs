mod controller;
mod shaders;
mod utils;
mod object_loader;
mod logger;
mod webgl;

use crate::controller::*;
use crate::shaders::*;
use crate::logger::*;
use crate::utils::*;
use crate::webgl::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;
use js_sys::Map;

#[wasm_bindgen]
pub fn initialize_web_gl(resources: Map) -> Result<(), JsValue> 
{
	set_panic_hook();

	//Register all required event listeners for interactivity
	register_get_mouse_position();

	rust_info(&"Initialising webgl...");
	let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("glCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap();
    let mut frame = WebGl2Frame 
	{
		context: canvas.get_context("webgl2")?.unwrap().dyn_into::<web_sys::WebGl2RenderingContext>()?,
		indices: Vec::new(),
		program: None
	};

	rust_info(&"Loading shaders to memory...");
    let vert_shader: &str = &(resources.get(&JsValue::from_str("vert_shader")).as_string().unwrap_or(String::from("bad_value")));
	rust_super_verbose(&("Vertex Shader is: ".to_owned() + &vert_shader));
    let frag_shader: &str = &(resources.get(&JsValue::from_str("frag_shader")).as_string().unwrap_or(String::from("bad_value")));
    rust_super_verbose(&("Fragment Shader is: ".to_owned() + &frag_shader));
	rust_info(&"...shaders load to memory complete.");

	rust_info(&"Compiling shaders...");
	let vert_shader = compile_shader(&frame, WebGl2RenderingContext::VERTEX_SHADER, vert_shader)?;
    let frag_shader = compile_shader(&frame, WebGl2RenderingContext::FRAGMENT_SHADER, frag_shader)?;
    rust_info(&"...shaders compilation complete.");

	rust_info(&"Linking shaders...");
	link_program(&mut frame, &vert_shader, &frag_shader)?;
    rust_info(&"...shaders linking complete");
	frame.context.use_program(frame.program.as_ref());

	rust_info(&"Loading scene to memory...");
	let scene: &str = &(resources.get(&JsValue::from_str("cube")).as_string().unwrap_or(String::from("bad_value")));
	rust_super_verbose(&("...Scene is:".to_owned() + &scene));

	rust_info(&"Loading textures to memory...");
    let mut textures: Vec<String> = Vec::new();
	textures.push(resources.get(&JsValue::from_str("texture")).as_string().unwrap_or(String::from("bad_value")));
	rust_info(&"...textures load to memory complete.");


	rust_verbose(&"Parsing scene...");
	let objset = match wavefront_obj::obj::parse(scene)
	{
		Ok(objset) => objset,
		Err(e) => panic!("{}", e)
	};
	rust_verbose(&"...scene parsing complete.");

	rust_info(&"...scene loading complete.");

	rust_info(&"Buffering scene to GPU...");
	webgl::buffer_scene(&mut frame, &objset, &textures)?;
	rust_info(&"...scene buffering complete.");

	//Pass context resolution for use in shader
	let resolution = get_window_resolution();
	rust_super_verbose
	(
		&(
			"Passing window resolution ".to_owned() + 
			resolution[0].to_string().as_str() + " x " + resolution[1].to_string().as_str() + 
			" to GPU"
		)
	);
	let resolution_index = frame.context.get_uniform_location(&frame.program.as_mut().unwrap(), "u_resolution");
	frame.context.uniform2fv_with_f32_array(resolution_index.as_ref(), &resolution);

	// Set up depth test
	rust_verbose(&"Configuring GPU depth testing...");
	webgl::enable_depthtest(&frame)?;
	rust_verbose(&"...configuration complete.");
	
	frame.context.clear_color(0.0, 0.0, 0.0, 0.0);
	rust_info(&"...webgl initialisation complete.");

	rust_info(&"Initializing animation loop...");
	webgl::initialize_animation(frame);
	rust_info(&"...animation loop initialisation complete.");

    return Ok(());
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