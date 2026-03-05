use crate::logger::*;
use crate::utils::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use webgl_matrix::*;
use js_sys::Map;

#[wasm_bindgen]
pub struct WebGl2Frame
{
	context: WebGl2RenderingContext,
	indices: Vec<u16>,
	program: Option<WebGlProgram>
} 

#[wasm_bindgen]
impl WebGl2Frame 
{
	#[wasm_bindgen(constructor)]
    pub fn new(resources: Map) -> Result<Self, JsValue> 
	{
        let document = web_sys::window().unwrap().document().unwrap();
    	let canvas = document.get_element_by_id("glCanvas").unwrap();
    	let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap();

		let mut frame = 
			Self 
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
		let vert_shader = frame.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_shader)?;
		let frag_shader = frame.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_shader)?;
		rust_info(&"...shaders compilation complete.");

		rust_info(&"Linking shaders...");
		frame.link_program(&vert_shader, &frag_shader)?;
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
		frame.buffer_scene(&objset, &textures)?;
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
		let resolution_index = frame.context.get_uniform_location(frame.program.as_mut().unwrap(), "u_resolution");
		frame.context.uniform2fv_with_f32_array(resolution_index.as_ref(), &resolution);

		// Set up depth test
		rust_verbose(&"Configuring GPU depth testing...");
		frame.enable_depthtest()?;
		rust_verbose(&"...configuration complete.");
		
		frame.context.clear_color(0.0, 0.0, 0.0, 0.0);

		rust_info(&"Initializing animation loop...");
		//frame.initialize_animation();
		rust_info(&"...animation loop initialisation complete.");

		return Ok(frame);
	}

	fn enable_depthtest(&self) -> Result<(), String>
	{
		self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
		self.context.depth_func(WebGl2RenderingContext::LESS);

		return Ok(());
	}

	fn draw(&self, context: &WebGl2RenderingContext, indices: &Vec<u16>) 
	{
		rust_super_super_verbose("Initiating draw call...");
		rust_super_super_verbose(&("drawing ".to_owned() + indices.len().to_string().as_str() + " indices"));
		context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
		context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, indices.len() as i32, WebGl2RenderingContext::UNSIGNED_SHORT, 0.0);
		rust_super_super_verbose("...draw call complete.");
	}

	fn window(&self) -> web_sys::Window 
	{
		web_sys::window().expect("no global `window` exists")
	}

	fn request_animation_frame(&mut self, f: &Closure<dyn FnMut()>) 
	{
		self.window()
			.request_animation_frame(f.as_ref().unchecked_ref())
			.expect("should register `requestAnimationFrame` OK");
	}

	/*
	*
	* Sets the projections matrix. Currently has no projection is hardcoded
	* TODO let user customise
	*
	*/
	pub fn set_projection(&self)
	{
		let projection_matrix = Mat4::create_perspective(1.0471975511965976, 0.8260869565217391, 1.0, 2000.0);
		let position_index = self.context.get_uniform_location(self.program.as_ref().unwrap(), "u_projection_matrix");
		self.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &projection_matrix);

		m4_pretty_print("Projection Matrix", &projection_matrix);
	}
}

mod shaders;
mod models;
pub mod animations;