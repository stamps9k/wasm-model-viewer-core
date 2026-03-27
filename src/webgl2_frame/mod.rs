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

		rust_info(&"Loading materials to memory...");
		let materials: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "materials");
		rust_info(&"...materials load to memory complete.");

		//rust_super_verbose(&("...materials are:".to_owned() + &materials));

		rust_info(&"Loading textures to memory...");
		let textures: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "textures");
		rust_info(&"...textures load to memory complete.");

		rust_verbose(&"Parsing scene...");
		let objset = match wavefront_obj::obj::parse(scene)
		{
			Ok(objset) => objset,
			Err(e) => panic!("{}", e)
		};
		rust_verbose(&"...scene parsing complete.");

		//If required parse the materials
		let mtls: Option<wavefront_obj::mtl::MtlSet> = None;
		if !materials.is_none()
		{
			rust_verbose(&"Parsing materials...");
			let material_text = materials.unwrap().into_values().next().expect("bad_value");
			let mtls = match wavefront_obj::mtl::parse(material_text)
			{
				Ok(mtls) => mtls,
				Err(e) => panic!("{}", e)
			};
			rust_verbose(&"...Materials parsing complete.");
			rust_info(&"...scene loading complete.");
		}

		rust_info(&"Buffering scene to GPU...");
		frame.buffer_scene(&objset, &mtls, &textures)?;
		rust_info(&"...scene buffering complete.");

		rust_super_verbose
		(
			&(
				"largest dimensions for model are ".to_owned() + 
				&frame.largest[0].to_string() + ", " +
				&frame.largest[1].to_string() + ", " +
				&frame.largest[2].to_string() + ", "
			)
		);

		rust_super_verbose
		(
			&(
				"smallest dimensions for model are ".to_owned() + 
				&frame.smallest[0].to_string() + ", " +
				&frame.smallest[1].to_string() + ", " +
				&frame.smallest[2].to_string() + ", "
			)
		);

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

		rust_info(&"Reseting the camera_matrix...");
		let mut central_matrix = Mat4::identity(); //Create the translation matrix to centralise object ontop of camera
		central_matrix.translate(&frame.get_centralisation()); //Create the translation matrix to centralise object ontop of camera
		let scale_mat: Mat4 = scaling_matrix(frame.get_scaling()); //Create the scaling matrix
		central_matrix.mul(&scale_mat); //Combine so that scaled model moves the right amount to sit on camera
		let mut translate_matrix = Mat4::identity(); //Create the translation matrix to pull the starting camera out of model
		translate_matrix.translate(&[0.0 as f32, 0.0 as f32, -5.0 as f32]); //Create the translation matrix to pull camera out of model
		frame.camera_matrix = *central_matrix.mul(&translate_matrix); //Combine with the operation S * T
		m4_pretty_print_super_verbose("Camera Matrix", &frame.camera_matrix);
		rust_info(&"...camera matrix reset complete.");

		return Ok(frame);
	}

	pub fn update_scene(&mut self, resources: Map) -> Result<(), JsValue>
	{
		//Cleaning up old scene
		for mut object in &mut self.objects
		{
			object.marked_for_deletion = true;
		}

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
		rust_info(&"...scene loading to memory complete.");

		rust_info(&"Loading materials to memory...");
		let materials: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "materials");
		rust_info(&"...materials load to memory complete.");

		//Check if texture is available and load to memory if relevant
		rust_info(&"Loading textures to memory...");
		let textures: Option<HashMap<String, String>> = get_js_sys_map_to_hashmap(&resources, "textures");
		rust_info(&"...textures load to memory complete.");

		rust_verbose(&"Parsing scene...");
		let objset = match wavefront_obj::obj::parse(scene)
		{
			Ok(objset) => objset,
			Err(e) => panic!("{}", e)
		};
		rust_verbose(&"...scene parsing complete.");

		//If required parse the materials
		let mtls: Option<wavefront_obj::mtl::MtlSet> = None;
		if !materials.is_none()
		{
			rust_verbose(&"Parsing materials...");
			let material_text = materials.unwrap().into_values().next().expect("bad_value");
			let mtls = match wavefront_obj::mtl::parse(material_text)
			{
				Ok(mtls) => mtls,
				Err(e) => panic!("{}", e)
			};
			rust_verbose(&"...Materials parsing complete.");
		}

		rust_info(&"...scene loading complete.");

		rust_info(&"Buffering scene to GPU...");
		self.buffer_scene(&objset, &mtls, &textures)?;
		rust_info(&"...scene buffering complete.");

		rust_verbose
		(
			&(
				"largest dimensions for model are ".to_owned() + 
				&self.largest[0].to_string() + ", " +
				&self.largest[1].to_string() + ", " +
				&self.largest[2].to_string() + ", "
			)
		);

		rust_verbose
		(
			&(
				"smallest dimensions for model are ".to_owned() + 
				&self.smallest[0].to_string() + ", " +
				&self.smallest[1].to_string() + ", " +
				&self.smallest[2].to_string() + ", "
			)
		);

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
		let mut central_matrix = Mat4::identity(); //Create the translation matrix to centralise object ontop of camera
		central_matrix.translate(&self.get_centralisation()); //Create the translation matrix to centralise object ontop of camera
		let scale_mat: Mat4 = scaling_matrix(self.get_scaling()); //Create the scaling matrix
		central_matrix.mul(&scale_mat); //Combine so that scaled model moves the right amount to sit on camera
		let mut translate_matrix = Mat4::identity(); //Create the translation matrix to pull the starting camera out of model
		translate_matrix.translate(&[0.0 as f32, 0.0 as f32, -5.0 as f32]); //Create the translation matrix to pull camera out of model
		self.camera_matrix = *central_matrix.mul(&translate_matrix); //Combine with the operation S * T
		m4_pretty_print_verbose("Camera Matrix", &self.camera_matrix);
		rust_info(&"...camera matrix reset complete.");

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
		// First pass - clean up any objects marked for deletion
		let context = &self.context;
		self.objects.retain_mut(|object| {
			if object.marked_for_deletion {
				rust_warn("Cleanup check");
				object.cleanup(context);
				false
			} else {
				true
			}
		});

		// Clear the color & depth buffers before drawing
		self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

		for n in 0..self.objects.len()
		{
			rust_super_super_verbose(&format!("Initiating draw call for object {}...", n));
			rust_super_verbose(&("drawing ".to_owned() + self.objects[n].indices_size.to_string().as_str() + " indices"));

			// If the object is untextured, just grab the position attribute for feeding from the model 
			if !self.objects[n].vertex_buffer.is_none()
			{
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].vertex_buffer.as_ref());
				let position_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_position") as u32;
				self.context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
				self.context.enable_vertex_attrib_array(position_attribute_location);
			// Else if there is a texture involved, grab the position and texture attributes
			} else if !self.objects[n].vertex_and_texture_buffer.is_none()
			{
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].vertex_and_texture_buffer.as_ref());				
				
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

				self.context.enable_vertex_attrib_array(position_attribute_location);
				self.context.enable_vertex_attrib_array(texture_attribute_location);
			}


			// Bind the color buffer and set the shader attribute for it to read to
			if !self.objects[n].color_buffer.is_none()
			{
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.objects[n].color_buffer.as_ref());
				let color_attribute_location = self.context.get_attrib_location(self.program.as_ref().unwrap(), "a_color") as u32;
				self.context.vertex_attrib_pointer_with_i32(color_attribute_location, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
				self.context.enable_vertex_attrib_array(color_attribute_location);
			}

			// Bind the vertex indices buffer
			if !self.objects[n].vertex_index_buffer.is_none()
			{
				self.context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.objects[n].vertex_index_buffer.as_ref());
			}

			// Finally draw
			self.context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, self.objects[n].indices_size as i32, WebGl2RenderingContext::UNSIGNED_SHORT, 0.0);
			
			rust_super_super_verbose("...draw call complete.");
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

	fn get_centralisation(&self) -> [f32; 3]
	{
		let middle_x: f32 = (self.largest[0] + self.smallest[0]) / 2.0;
		let middle_y: f32 = (self.largest[1] + self.smallest[1]) / 2.0;
		let middle_z: f32 = (self.largest[2] + self.smallest[2]) / 2.0;

		return [-middle_x, -middle_y, -middle_z];
	}
}

mod shaders;
mod models;
pub mod animations;