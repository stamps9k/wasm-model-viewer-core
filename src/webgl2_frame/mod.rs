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
	program: Option<WebGlProgram>,
	largest: [f32; 3],
    smallest: [f32; 3],
	camera_matrix: Mat4
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
				program: None,
				largest: [0.0, 0.0, 0.0],
    			smallest: [0.0, 0.0, 0.0],
				camera_matrix: Mat4::identity()
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

	pub fn update_scene(&mut self, resources: Map) -> Result<(), JsValue>
	{
		rust_info(&"Loading shaders to memory...");
		let vert_shader: &str = &(resources.get(&JsValue::from_str("vert_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_super_verbose(&("Vertex Shader is: ".to_owned() + &vert_shader));
		let frag_shader: &str = &(resources.get(&JsValue::from_str("frag_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_super_verbose(&("Fragment Shader is: ".to_owned() + &frag_shader));
		rust_info(&"...shaders load to memory complete.");

		rust_info(&"Compiling shaders...");
		let vert_shader = self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_shader)?;
		let frag_shader = self.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_shader)?;
		rust_info(&"...shaders compilation complete.");

		rust_info(&"Linking shaders...");
		self.link_program(&vert_shader, &frag_shader)?;
		rust_info(&"...shaders linking complete");
		self.context.use_program(self.program.as_ref());

		rust_info(&"Loading scene to memory...");
		let scene: &str = &(resources.get(&JsValue::from_str("cube")).as_string().unwrap_or(String::from("bad_value")));
		rust_super_verbose(&("...Scene is:".to_owned() + &scene));

		let mut textures: Vec<String> = Vec::new();

		//Check if texture is available and load to memory if relevant
		if resources.get(&JsValue::from_str("texture")) != JsValue::NULL
		{
			rust_info(&"Loading textures to memory...");
			textures.push(resources.get(&JsValue::from_str("texture")).as_string().unwrap_or(String::from("bad_value")));
			rust_info(&"...textures load to memory complete.");
		} 
		else 
		{
			textures.push(String::from("bad_value"));
		}

		rust_verbose(&"Parsing scene...");
		let objset = match wavefront_obj::obj::parse(scene)
		{
			Ok(objset) => objset,
			Err(e) => panic!("{}", e)
		};
		rust_verbose(&"...scene parsing complete.");
		rust_info(&"...scene loading complete.");

		rust_info(&"Buffering scene to GPU...");
		self.buffer_scene(&objset, &textures)?;
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
		let resolution_index = self.context.get_uniform_location(self.program.as_mut().unwrap(), "u_resolution");
		self.context.uniform2fv_with_f32_array(resolution_index.as_ref(), &resolution);

		// Set up depth test
		rust_verbose(&"Configuring GPU depth testing...");
		self.enable_depthtest()?;
		rust_verbose(&"...configuration complete.");
		
		self.context.clear_color(0.0, 0.0, 0.0, 0.0);

		rust_info(&"Reseting the camera_matrix...");
		self.camera_matrix = Mat4::identity();
		self.camera_matrix.translate(&[0.0 as f32, 0.0 as f32, -10.0 as f32]);
		self.camera_matrix.scale(self.get_scaling());
		m4_pretty_print_super_verbose("Camera Matrix", &self.camera_matrix);
		rust_info(&"...camera matrix reset complete.");

		return Ok(());	
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
		rust_info(&("Setting the projection matrix. Currently hard coded...".to_owned()));
		let projection_matrix = Mat4::create_perspective(1.0471975511965976, 0.8260869565217391, 1.0, 2000.0);
		let position_index = self.context.get_uniform_location(self.program.as_ref().unwrap(), "u_projection_matrix");
		self.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &projection_matrix);
		rust_info(&("...projection matri successfully set.".to_owned()));

		m4_pretty_print_super_verbose("Projection Matrix", &projection_matrix);
	}

	/*
	*
	* Get the scalling for the given scene. Done so that models always starts at a reasonable size
	*
	*/
	fn get_scaling(&self) -> f32
	{
		let dimension_diff: [f32; 3] = 
		[
			self.largest[0] - self.smallest[0],
			self.largest[1] - self.smallest[1],
			self.largest[2] - self.smallest[2]
		];
		rust_info(&("Difference across each dimension is: ".to_owned() + &(dimension_diff[0].to_string()) + ", " + &(dimension_diff[1].to_string()) + ", " + &(dimension_diff[2].to_string())));

		let mut largest_dimension: f32 = 0.0;

		for n in dimension_diff
		{
			if n > largest_dimension
			{
				largest_dimension = n;
			}
		}
		rust_info(&("Largest difference is ".to_owned() + &(largest_dimension.to_string())));

		let scale: f32 = 1.0 / largest_dimension;
		rust_info(&("Scaling factor calculated to be ".to_owned() + &(scale.to_string())));

		return scale;
	}
}

mod shaders;
mod models;
mod object_loader;
pub mod animations;