use crate::controller::*;
use crate::logger::*;
use crate::object_loader;
use crate::utils::*;

use math::mean;
use rand::Rng;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use webgl_matrix::*;
use wavefront_obj::obj::ObjSet;
use wavefront_obj::obj::Object;
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

	fn buffer_scene(&mut self, objset: &ObjSet, textures: &Vec<String>) -> Result<(), String>
	{
		rust_info(&"Loading textures to memory...");
		rust_super_verbose(&("Texture is: ".to_owned() + &textures[0]));
		rust_info(&"... texture loading complete.");

		for n in 0..(&objset).objects.len()
		{
			//Ignore junk objects
			if (&objset).objects[n].vertices.len() != 0	
			{
				rust_info(&("Buffering model ".to_owned() + n.to_string().as_str() + ": " + &objset.objects[n].name + "to GPU..."));
				rust_info(&(textures.len().to_string().as_str()));
				//TODO Properly generate and pass in textures.
				self.buffer_obj(&objset.objects[n], textures[0].clone())?;
				rust_info(&"...model buffering complete.");
			}
		}

		self.set_projection();

		return Ok(());
	}

	fn enable_depthtest(&self) -> Result<(), String>
	{
		self.context.enable(WebGl2RenderingContext::DEPTH_TEST);
		self.context.depth_func(WebGl2RenderingContext::LESS);

		return Ok(());
	}

	fn buffer_obj(&mut self, obj: &Object, texture_b64: String) -> Result<(), String>
	{
		/*
		*
		*	Get all relevant infomation from the wavefront object
		*
		*/
		let vertex_positions: Vec<f32> = object_loader::get_vertex_positions(&obj);
		rust_verbose(&("Vertices only is size: ".to_owned() + vertex_positions.len().to_string().as_str()));
		let vertex_indices: Vec<u16> = object_loader::get_vertex_indices(&obj);
		rust_verbose(&("Vertex Indices is size: ".to_owned() + vertex_indices.len().to_string().as_str()));

		let texture_vertices: Vec<f32> = object_loader::get_texture_positions(&obj);
		rust_verbose(&("Texutre Vertices is size: ".to_owned() + texture_vertices.len().to_string().as_str()));
		let texture_indices: Vec<u16> = object_loader::get_texture_indices(&obj);
		rust_verbose(&("Texutre Indices is size: ".to_owned() + texture_indices.len().to_string().as_str()));
		
		/*
			Create the vertex array object	
		*/
		rust_verbose(&"Creating vertex array object...");
		let vao = self.context.create_vertex_array();
		self.context.bind_vertex_array(vao.as_ref());
		rust_verbose(&"...vertex array object creation completed.");

		rust_verbose(&("Object: ".to_owned() + obj.name.as_str() + "identified as textured model. Processing accordingly"));
		if texture_vertices.len() > 0 
		{
			/*
				Manage model texture and vertices
			*/
				// First generate the texture and vertex info
				rust_verbose("Generating a list that has all unique combined vertex + texture positions...");
				let merged_array: Vec<f32> = object_loader::merge_vertex_and_texture_positions(&vertex_positions, &vertex_indices, &texture_vertices, &texture_indices);
				rust_verbose("...combined vertex + texture positions list completed.");

				// create the GPU buffer
				rust_verbose("Creating  GPU buffer for vertex and texture positions array...");
				let vertex_and_texture_buffer = self.context.create_buffer().ok_or("failed to create a buffer for textures")?;
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_and_texture_buffer));
				rust_verbose("...GPU buffer creation complete.");

				//Put values into buffer
				rust_verbose(&("Starting to buffer vertex & texture locations... "));
				unsafe {
					let texture_coord_array = js_sys::Float32Array::view(&merged_array);
				
					self.context.buffer_data_with_array_buffer_view
					(
						WebGl2RenderingContext::ARRAY_BUFFER,
						&texture_coord_array,
						WebGl2RenderingContext::STATIC_DRAW
					);
				}
				rust_verbose(&("...buffering complete."));

				//Tell GPU how to extract vertex data from the buffer
				let position_attribute_location = self.context.get_attrib_location(self.program.as_mut().unwrap(), "a_position") as u32;
				self.context.vertex_attrib_pointer_with_i32
				(
					position_attribute_location, //index
					3, //size
					WebGl2RenderingContext::FLOAT, //data type
					false, //normalized
					20, //stride
					0 //offset
				);

				//Tell GPU how to extract texture data from the buffer
				let texture_attribute_location = self.context.get_attrib_location(self.program.as_mut().unwrap(), "a_texcoord") as u32;
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
				
				//Buffer the texture image
				rust_verbose(&("Starting to buffer texture image... "));
				let texture = self.context.create_texture().ok_or("failed to create texture")?;
				self.context.active_texture(WebGl2RenderingContext::TEXTURE0);
				self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
				let image = object_loader::create_image_as_uint8_array(texture_b64.as_str())?;
				self.context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
				let _ = 
				match 
					self.context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_js_u8_array
					(
						WebGl2RenderingContext::TEXTURE_2D,
						0,
						WebGl2RenderingContext::RGBA8 as i32,
						320,
						320,
						0,
						WebGl2RenderingContext::RGBA, // format
						WebGl2RenderingContext::UNSIGNED_BYTE, // type
						Some(&image)
					)
				{
					Ok(result) => result,
					Err(_err) => panic!("failed to send image data to texture buffer.")
				};
				self.context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
				rust_verbose(&("...texture buffering complete."));

			/*
			Manage Indices for model
			*/
			rust_verbose(&("Starting to buffer vertex & texture position indices... "));
			let vert_index = self.context.create_buffer().ok_or("failed to create buffer")?;
			self.context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&vert_index));
			unsafe {
				self.indices = (0..(merged_array.len() / 5) as u16).collect();
				let converted_indices = js_sys::Uint16Array::view(&self.indices);
				rust_super_verbose(&(converted_indices.to_string().as_string().unwrap()));
				self.context.buffer_data_with_array_buffer_view(
					WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
					&converted_indices,
					WebGl2RenderingContext::STATIC_DRAW,
				);
			}
			rust_verbose(&("...buffering complete."));
		} else {
			/*
				Manage Vertices for model
			
				Note that `Float32Array::view` is somewhat dangerous (hence the
			`unsafe`!). This is creating a raw view into our module's
				`WebAssembly.Memory` buffer, but if we allocate more pages for ourself
			(aka do a memory allocation in Rust) it'll cause the buffer to change,
			causing the `Float32Array` to be invalid.
				
			As a result, after `Float32Array::view` we have to be very careful not to
			do any memory allocations before it's dropped.
			*/
			unsafe 
			{
				rust_verbose(&("Starting to buffer vertex data... "));
				let vert_buffer = self.context.create_buffer().ok_or("failed to create buffer")?;
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vert_buffer));

				let position_attribute_location = self.context.get_attrib_location(self.program.as_mut().unwrap(), "a_position") as u32;
				self.context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);	
						
				let vert_array = js_sys::Float32Array::view(&vertex_positions);
				self.context.buffer_data_with_array_buffer_view
				(
					WebGl2RenderingContext::ARRAY_BUFFER,
					&vert_array,
					WebGl2RenderingContext::STATIC_DRAW,
				);
				self.context.enable_vertex_attrib_array(position_attribute_location);
				rust_verbose(&("..Vertex data fully buffered."));
			}

			/*
				Manage Colors for model
			*/
			unsafe {
				rust_verbose(&("Starting to buffer color data... "));
				let color_buffer = self.context.create_buffer().ok_or("failed to create buffer")?;
				self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
				self.context.vertex_attrib_pointer_with_i32(1, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
				self.context.enable_vertex_attrib_array(1);
		
				//Currently junk colors. Only care about matching vertex count in sample cube
				let mut rng = rand::rng();
				let mut colors: Vec<f32> = Vec::new();
				for n in 0..vertex_indices.len()
				{
					if n % 3 == 0
					{
						let c: f32 = rng.random_range(0.0..=1.0);
						colors.push(c);
						colors.push(c);
						colors.push(c);
					}

				}
				let color_array = js_sys::Float32Array::view(&colors);

				self.context.buffer_data_with_array_buffer_view
				(
					WebGl2RenderingContext::ARRAY_BUFFER,
					&color_array,
					WebGl2RenderingContext::STATIC_DRAW,
				);
				rust_verbose(&("...color data buffering complete."));
			}

			/*
			Manage Indices for model
			*/
			rust_verbose(&("Starting to buffer vertex indices... "));
			let vert_index = self.context.create_buffer().ok_or("failed to create buffer")?;
			self.context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&vert_index));
			unsafe {	 
				self.indices = object_loader::get_vertex_indices(&obj);
				let converted_indices = js_sys::Uint16Array::view(&self.indices);
			
				self.context.buffer_data_with_array_buffer_view(
					WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
					&converted_indices,
					WebGl2RenderingContext::STATIC_DRAW,
				);
			}
			rust_verbose(&("..indice buffering complete."));

		}
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

pub fn initialize_animation(frame_wrap: &Rc<RefCell<WebGl2Frame>>) 
{
	//Clone the frame for the closure
	let frame_closure = Rc::clone(frame_wrap);

	//Clone the frame for the intial animation frame request
	let frame_animation = Rc::clone(frame_wrap);

	//Closure variables
	let f = Rc::new(RefCell::new(None));
	let g = f.clone();

	//FPS calculator variables
	let mut base: f64 = get_current_time();
	let mut frames_delta: [f64; 10] = [0.0; 10];

	//Initial time tracker
	let mut time: f32 = 1.0;

	//Initial Camera Matrix
	let mut camera_matrix = Mat4::identity();
	camera_matrix.translate(&[0.0 as f32, 0.0 as f32, -10.0 as f32]);

	let mut i: f32 = 0.0;
	*g.borrow_mut() = Some(Closure::new(move || {	
		//Movement variables
		let tmp2 = get_control_flags();
		let controller_values = tmp2.lock().unwrap();
		
		//FPS Caclulator
		let now = get_current_time();
		match i as i32 % 10 
		{
			0 => frames_delta[0] = now - base,
			1 => frames_delta[1] = now - base,
			2 => frames_delta[2] = now - base,
			3 => frames_delta[3] = now - base,
			4 => frames_delta[4] = now - base,
			5 => frames_delta[5] = now - base,
			6 => frames_delta[6] = now - base,
			7 => frames_delta[7] = now - base,
			8 => frames_delta[8] = now - base,
			9 => 
			{
				frames_delta[9] = now - base;
				base = get_current_time();
				let fps: f64 = mean::arithmetic(&frames_delta);
				set_fps(fps);
			},
			_ => panic!("Don't know how you got here!")
		}

		camera_matrix = update_camera_position(&camera_matrix, &controller_values);

		{
			//Borrow the Rc as a mutable for use in the animation
			let mut frame = frame_closure.borrow_mut();

			//Mutable reference to the Webgl Frame
			let tmp = frame.program.as_mut().unwrap().clone();

			//Pass worldspace transfomration to the GPU
			let position_index = frame.context.get_uniform_location(&tmp, "u_camera_matrix");
			frame.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &camera_matrix);
			
			//Pass mouse position to the GPU
			let mouse_position = controller_values.mouse_position;
			rust_super_super_verbose
			(
				&(
					"Passing u_mouse_position ".to_owned() + 
					mouse_position[0].to_string().as_str() + " x " + mouse_position[1].to_string().as_str() + 
					" to shader"
				)
			);
			let mouse_position_index = frame.context.get_uniform_location(&tmp, "u_mouse_position");
			frame.context.uniform2fv_with_f32_array(mouse_position_index.as_ref(), &mouse_position);

			//Pass time to the GPU
			time = time + ((now - base) / 1000.0) as f32;
			rust_super_super_verbose(&("Passing u_time ".to_owned() + time.to_string().as_str() + " to shader."));
			let time_index = frame.context.get_uniform_location(&tmp, "u_time");
			frame.context.uniform1f(time_index.as_ref(), time);

			frame.draw(&frame.context, &frame.indices);

		

		// Set the body's text content to how many times this
		// requestAnimationFrame callback has fired.
		i += 1.0;

			// Schedule ourself for another requestAnimationFrame callback.
			frame.request_animation_frame(f.borrow().as_ref().unwrap());
		}
	}));

	frame_animation.borrow_mut().request_animation_frame(g.borrow().as_ref().unwrap());
}

mod shaders;