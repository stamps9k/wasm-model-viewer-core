use crate::logger::*;
use crate::utils::*;
use crate::matrix_helper::*;
use crate::webgl2_wavefront_object::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use webgl_matrix::*;
use js_sys::Map;
use std::collections::HashMap;

#[wasm_bindgen]
pub struct WebGl2Frame
{
	context: WebGl2RenderingContext,
	program: Option<WebGlProgram>,
	objects: Vec<WebGl2WavefrontObject>,
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
				program: None,
				objects: Vec::new(),
				largest: [0.0, 0.0, 0.0],
    			smallest: [0.0, 0.0, 0.0],
				camera_matrix: Mat4::identity()
			};

		rust_log(&"Loading shaders to memory...", &"info_wasm_parse");
		let vert_shader: &str = &(resources.get(&JsValue::from_str("vert_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&format!("Vertex Shader is: {}", vert_shader), &"verbose_wasm_parse");
		let frag_shader: &str = &(resources.get(&JsValue::from_str("frag_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&format!("Fragment Shader is: {}", frag_shader), &"verbose_wasm_parse");
		rust_log(&"...shaders load to memory complete.", &"info_wasm_parse");

		rust_log(&"Compiling shaders...", &"info_wasm_gpu_mem");
		let vert_shader = frame.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_shader)?;
		let frag_shader = frame.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_shader)?;
		rust_log(&"...shaders compilation complete.", &"info_wasm_gpu_mem");

		rust_log(&"Linking shaders...", &"info_wasm_gpu_data");
		frame.link_program(&vert_shader, &frag_shader)?;
		frame.context.use_program(frame.program.as_ref());
		rust_log(&"...shaders linking complete", &"info_wasm_gpu_data");

		rust_log(&"Loading scene to memory...", &"info_wasm_parse");
		let scene: &str = &(resources.get(&JsValue::from_str("cube")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&"...loading complete.", &"info_wasm_parse");

		rust_log(&"Loading materials to memory...", &"info_wasm_parse");
		let materials: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "materials");
		rust_log(&"...materials load to memory complete.", &"info_wasm_parse");

		rust_log(&"Loading textures to memory...", &"info_wasm_parse");
		let textures: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "textures");
		rust_log(&"...textures load to memory complete.", &"info_wasm_parse");

		rust_log(&"Parsing scene...", &"info_wasm_parse");
		let objset = match wavefront_obj::obj::parse(scene)
		{
			Ok(objset) => objset,
			Err(e) => panic!("{}", e)
		};
		rust_log(&"...scene parsing complete.", &"info_wasm_parse");

		//If required parse the materials
		let mtls: Option<wavefront_obj::mtl::MtlSet> = None;
		if !materials.is_none()
		{
			rust_log(&"Parsing materials...", &"verbose_wasm_parse");
			let material_text = materials.unwrap().into_values().next().expect("bad_value");
			let mtls = match wavefront_obj::mtl::parse(material_text)
			{
				Ok(mtls) => mtls,
				Err(e) => panic!("{}", e)
			};
			rust_log(&"...Materials parsing complete.", &"verbose_wasm_parse");
		}
		rust_log(&"...scene loading complete.", &"verbose_wasm_scene");

		rust_log(&"Buffering scene to GPU...", &"verbose_wasm_gpu_mem");
		frame.buffer_scene(&objset, &mtls, &textures)?;
		rust_log(&"...scene buffering complete.", &"verbose_wasm_gpu_mem");

		//Pass context resolution for use in shader
		let resolution = get_window_resolution();
		rust_log(&format!("Passing window resolution {} x {} to gpu.", resolution[0], resolution[1]), &"verbose_wasm_scene");

		let resolution_index = frame.context.get_uniform_location(frame.program.as_mut().unwrap(), "u_resolution");
		frame.context.uniform2fv_with_f32_array(resolution_index.as_ref(), &resolution);

		// Set up depth test
		rust_log(&"Configuring GPU depth testing...", &"verbose_wasm_gpu_mem");
		frame.enable_depthtest()?;
		frame.context.clear_color(0.0, 0.0, 0.0, 0.0);
		rust_log(&"...configuration complete.", &"verbose_wasm_gpu_mem");		

		rust_log(&"Reseting the camera_matrix...", &"verbose_wasm_math");
		let mut central_matrix = Mat4::identity(); //Create the translation matrix to centralise object ontop of camera
		central_matrix.translate(&frame.get_centralisation()); //Create the translation matrix to centralise object ontop of camera
		let scale_mat: Mat4 = scaling_matrix(frame.get_scaling()); //Create the scaling matrix
		central_matrix.mul(&scale_mat); //Combine so that scaled model moves the right amount to sit on camera
		let mut translate_matrix = Mat4::identity(); //Create the translation matrix to pull the starting camera out of model
		translate_matrix.translate(&[0.0 as f32, 0.0 as f32, -5.0 as f32]); //Create the translation matrix to pull camera out of model
		frame.camera_matrix = *central_matrix.mul(&translate_matrix); //Combine with the operation S * T
		m4_pretty_print_super_verbose("Camera Matrix", &frame.camera_matrix);
		rust_log(&"...camera matrix reset complete.", &"verbose_wasm_math");

		return Ok(frame);
	}

	pub fn update_scene(&mut self, resources: Map) -> Result<(), JsValue>
	{
		//Cleaning up old scene
		for mut object in &mut self.objects
		{
			object.marked_for_deletion = true;
			self.largest = [0.0, 0.0, 0.0];
			self.smallest = [0.0, 0.0, 0.0];
		}

		rust_log(&"Loading shaders to memory...", &"info_wasm_parse");
		let vert_shader: &str = &(resources.get(&JsValue::from_str("vert_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&format!("Vertex shader is: {}", vert_shader), &"super_verbose_wasm_parse");
		let frag_shader: &str = &(resources.get(&JsValue::from_str("frag_shader")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&format!("Fragement shader is: {}", frag_shader), &"super_verbose_wasm_parse");
		rust_log(&"...shaders load to memory complete.", &"info_wasm_parse");

		rust_log(&"Compiling shaders...", &"info_wasm_gpu_mem");
		let vert_shader = self.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, vert_shader)?;
		let frag_shader = self.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, frag_shader)?;
		rust_log(&"...shaders compilation complete.", &"info_wasm_gpu_mem");

		rust_log(&"Linking shaders...", &"info_wasm_gpu_data");
		self.link_program(&vert_shader, &frag_shader)?;
		self.context.use_program(self.program.as_ref());
		rust_log(&"...shaders linking complete", &"info_wasm_gpu_data");

		rust_log(&"Loading scene to memory...", &"info_wasm_scene");
		let scene: &str = &(resources.get(&JsValue::from_str("cube")).as_string().unwrap_or(String::from("bad_value")));
		rust_log(&"...scene loading to memory complete.", &"info_wasm_scene");

		rust_log(&"Loading materials to memory...", &"info_wasm_scene");
		let materials: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "materials");
		rust_log(&"...materials load to memory complete.", &"info_wasm_scene");

		//Check if texture is available and load to memory if relevant
		rust_log(&"Loading textures to memory...", &"info_wasm_scene");
		let textures: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "textures");
		rust_log(&"...textures load to memory complete.", &"info_wasm_scene");

		rust_log(&"Parsing scene...", &"info_wasm_parse");
		let objset = match wavefront_obj::obj::parse(scene)
		{
			Ok(objset) => objset,
			Err(e) => panic!("{}", e)
		};
		rust_log(&"...scene parsing complete.", &"info_wasm_parse");

		//If required parse the materials
		let mtls: Option<wavefront_obj::mtl::MtlSet> = None;
		if !materials.is_none()
		{
			rust_log(&"Parsing materials...", &"info_wasm_parse");
			let material_text = materials.unwrap().into_values().next().expect("bad_value");
			let mtls = match wavefront_obj::mtl::parse(material_text)
			{
				Ok(mtls) => mtls,
				Err(e) => panic!("{}", e)
			};
			rust_log(&"...Materials parsing complete.", &"info_wasm_parse");
		}

		rust_log(&"...scene loading complete.", &"info_wasm_scene");

		rust_log(&"Buffering scene to GPU...", &"info_wasm_parse");
		self.buffer_scene(&objset, &mtls, &textures)?;
		rust_log(&"...scene buffering complete.", &"info_wasm_parse");

		//Pass context resolution for use in shader
		let resolution = get_window_resolution();
		rust_log(&format!("Passing window resolution {} x {} to gpu.", resolution[0], resolution[1]), &"verbose_wasm_scene");

		let resolution_index = self.context.get_uniform_location(self.program.as_mut().unwrap(), "u_resolution");
		self.context.uniform2fv_with_f32_array(resolution_index.as_ref(), &resolution);

		// Set up depth test
		rust_log(&"Configuring GPU depth testing...", &"info_wasm_gpu_data");
		self.enable_depthtest()?;
		rust_log(&"...configuration complete.", &"info_wasm_gpu_data");
		
		self.context.clear_color(0.0, 0.0, 0.0, 0.0);

		rust_log(&"Reseting the camera_matrix...", &"info_wasm_math");
		let mut central_matrix = Mat4::identity(); //Create the translation matrix to centralise object ontop of camera
		central_matrix.translate(&self.get_centralisation()); //Create the translation matrix to centralise object ontop of camera
		let scale_mat: Mat4 = scaling_matrix(self.get_scaling()); //Create the scaling matrix
		central_matrix.mul(&scale_mat); //Combine so that scaled model moves the right amount to sit on camera
		let mut translate_matrix = Mat4::identity(); //Create the translation matrix to pull the starting camera out of model
		translate_matrix.translate(&[0.0 as f32, 0.0 as f32, -5.0 as f32]); //Create the translation matrix to pull camera out of model
		self.camera_matrix = *central_matrix.mul(&translate_matrix); //Combine with the operation S * T
		m4_pretty_print_verbose("Camera Matrix", &self.camera_matrix);
		rust_log(&"...camera matrix reset complete.", &"info_wasm_math");

		return Ok(());	
	}

	fn enable_depthtest(&self) -> Result<(), String>
	{
		self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
		self.context.depth_func(WebGl2RenderingContext::LESS);

		return Ok(());
	}

	fn draw(&mut self) 
	{
		rust_log(&format!("Cleaning up objects marked for deletion..."), &"super_verbose_wasm_scene");
		// First pass - clean up any objects marked for deletion
		let context = &self.context;
		self.objects.retain_mut(|object| {
			if object.marked_for_deletion {
				object.cleanup(context);
				false
			} else {
				true
			}
		});
		rust_log(&format!("...clean up complete"), &"super_verbose_wasm_scene");

		// Clear the color & depth buffers before drawing
		self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

		for n in 0..self.objects.len()
		{
			rust_log(&format!("Initiating draw call for object {}...", n), &"super_verbose_wasm_scene");

			rust_log(&format!("Drawing {} indices...", self.objects[n].indices_size), &"super_verbose_wasm_parse");
			// If the object is untextured, just grab the position attribute for feeding from the model 
			if !self.objects[n].vertex_buffer.is_none()
			{
				rust_log("UNTEXTURED OBJECT. Binding position data...", &"super_verbose_gpu_data");
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].vertex_buffer.as_ref());
				let position_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_position") as u32;
				self.context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
				self.context.enable_vertex_attrib_array(position_attribute_location);
				rust_log("...position data binding complete.", &"super_verbose_gpu_data");
			// Else if there is a texture involved, grab the position and texture attributes
			} 
			else if !self.objects[n].vertex_and_texture_buffer.is_none()
			{
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].vertex_and_texture_buffer.as_ref());				
				
				rust_log("TEXTURED OBJECT. Binding position data...", &"super_verbose_gpu_data");
				let position_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_position") as u32;
				self.context.vertex_attrib_pointer_with_i32
				(
					position_attribute_location, //index
					3, //size
					WebGl2RenderingContext::FLOAT, //data type
					false, //normalized
					20, //stride
					0 //offset
				);
				rust_log("...position data binding complete.", &"super_verbose_gpu_data");

				rust_log("TEXTURED OBJECT. Binding texture data...", &"super_verbose_gpu_data");				
				let texture_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_texcoord") as u32;
				self.context.vertex_attrib_pointer_with_i32
				(
					texture_attribute_location, //index
					2, //size
					WebGl2RenderingContext::FLOAT, //data type
					false, //normalized 
					20, //stride
					12 //offset
				);
				rust_log("...texture data binding complete.", &"super_verbose_gpu_data");

				self.context.enable_vertex_attrib_array(position_attribute_location);
				self.context.enable_vertex_attrib_array(texture_attribute_location);
			}


			// Bind the color buffer and set the shader attribute for it to read to
			if !self.objects[n].color_buffer.is_none()
			{
				rust_log("UNTEXTURED OBJECT. Binding color data...", &"super_verbose_gpu_data");
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].color_buffer.as_ref());
				let color_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_color") as u32;
				self.context.vertex_attrib_pointer_with_i32(color_attribute_location, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
				self.context.enable_vertex_attrib_array(color_attribute_location);
				rust_log("...color data binding complete.", &"super_verbose_gpu_data");
			}

			// Bind the vertex indices buffer
			if !self.objects[n].vertex_index_buffer.is_none()
			{
				rust_log("Binding index data...", &"super_verbose_gpu_data");
				self.context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.objects[n].vertex_index_buffer.as_ref());
				rust_log("...index data binding complete.", &"super_verbose_gpu_data");
			}

			// Finally draw
			self.context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, self.objects[n].indices_size as i32, WebGl2RenderingContext::UNSIGNED_SHORT, 0.0);
			rust_log(&format!("...{} indices drawn.", self.objects[n].indices_size), &"super_verbose_wasm_parse");

			rust_log(&format!("...drawing of object {} complete.", n), &"super_verbose_wasm_scene");
		}
		
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
		rust_log("Setting the projection matrix. Currently hard coded...", &"verbose_wasm_math");
		let projection_matrix = Mat4::create_perspective(1.0471975511965976, 0.8260869565217391, 1.0, 2000.0);
		let position_index = self.context.get_uniform_location(self.program.as_ref().unwrap(), "u_projection_matrix");
		self.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &projection_matrix);
		rust_log("...projection matrix successfully set.", &"verbose_wasm_math");

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

		rust_log(&format!("Difference across each dimension is: {}, {}, {}", dimension_diff[0], dimension_diff[1], dimension_diff[2]), &"verbose_wasm_math");

		let mut largest_dimension: f32 = 0.0;

		for n in dimension_diff
		{
			if n > largest_dimension
			{
				largest_dimension = n;
			}
		}
		rust_log(&format!("Largest difference is: {}", largest_dimension), &"verbose_wasm_math");

		let scale: f32 = 1.0 / largest_dimension;
		rust_log(&format!("Scaling factor calculated to be: {}", scale), &"verbose_wasm_math");

		return scale;
	}

	fn get_centralisation(&self) -> [f32; 3]
	{
		let middle_x: f32 = (self.largest[0] + self.smallest[0]) / 2.0;
		let middle_y: f32 = (self.largest[1] + self.smallest[1]) / 2.0;
		let middle_z: f32 = (self.largest[2] + self.smallest[2]) / 2.0;

		rust_log(&format!("Translation to centralize the object is: {}, {}, {}", middle_x, middle_y, middle_z), &"verbose_wasm_math");

		return [-middle_x, -middle_y, -middle_z];
	}
}

mod shaders;
mod models;
pub mod animations;