use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use web_sys::WebGlBuffer;
//use wasm_bindgen::prelude::*;
//use wasm_bindgen::JsCast;
use rand::Rng;
use std::collections::HashMap;


use crate::logger::*;

pub struct WebGl2WavefrontObject
{
	obj: wavefront_obj::obj::Object,
	mtls: Option<wavefront_obj::mtl::MtlSet>,
	pub vertex_buffer: Option<WebGlBuffer>,
	pub vertex_and_texture_buffer: Option<WebGlBuffer>,
	pub vertex_index_buffer: Option<WebGlBuffer>,
    pub indices_size: usize,
	pub color_buffer: Option<WebGlBuffer>,
    textures: Option<HashMap<String, String>>,
	pub texture_height: i32,
	pub texture_width: i32,
	pub largest: [f32; 3],
	pub smallest: [f32; 3]
} 

impl WebGl2WavefrontObject
{
    pub fn new(obj: wavefront_obj::obj::Object, mtls: Option<wavefront_obj::mtl::MtlSet>, textures: Option<HashMap<String, String>>) -> Result<Self, String>
    {
        let object = Self {
            obj: obj,
			mtls: mtls,
			vertex_buffer: None,
			vertex_and_texture_buffer: None,
			vertex_index_buffer: None,
			indices_size: 0,
			color_buffer: None,
            textures: textures,
			texture_height: 0,
			texture_width: 0,
			largest: [0.0, 0.0, 0.0],
			smallest: [0.0, 0.0, 0.0]
        };

        return Ok(object);
    }

    pub fn buffer(&mut self, context: &WebGl2RenderingContext, program: &Option<WebGlProgram>) -> Result<(), String>
	{
		/*
		*
		*	Get all relevant infomation from the wavefront object
		*
		*/
		let vertex_positions: Vec<f32> = self.get_vertex_positions();
		rust_verbose(&("Vertices only is size: ".to_owned() + vertex_positions.len().to_string().as_str()));
		let vertex_indices: Vec<u16> = self.get_vertex_indices();
		rust_verbose(&("Vertex Indices is size: ".to_owned() + vertex_indices.len().to_string().as_str()));

		let texture_vertices: Vec<f32> = self.get_texture_positions();
		rust_verbose(&("Texutre Vertices is size: ".to_owned() + texture_vertices.len().to_string().as_str()));
		let texture_indices: Vec<u16> = self.get_texture_indices();
		rust_verbose(&("Texutre Indices is size: ".to_owned() + texture_indices.len().to_string().as_str()));
		
		/*
			Create the vertex array object	
		*/
		rust_verbose(&"Creating vertex array object...");
		let vao = context.create_vertex_array();
		context.bind_vertex_array(vao.as_ref());
		rust_verbose(&"...vertex array object creation completed.");

		if self.textures != None
		{
			rust_verbose(&("Object: ".to_owned() + self.obj.name.as_str() + " identified as textured model. Processing accordingly"));
			
			/*
				Manage model texture and vertices
			*/
				// First generate the texture and vertex info
				rust_verbose("Generating a list that has all unique combined vertex + texture positions...");
				let merged_array: Vec<f32> = self.merge_vertex_and_texture_positions(&vertex_positions, &vertex_indices, &texture_vertices, &texture_indices);
				rust_verbose("...combined vertex + texture positions list completed.");

				// create the GPU buffer
				rust_verbose("Creating  GPU buffer for vertex and texture positions array...");
				self.vertex_and_texture_buffer = context.create_buffer();
				context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_and_texture_buffer.as_ref());
				rust_verbose("...GPU buffer creation complete.");

				//Put values into buffer
				rust_verbose(&("Starting to buffer vertex & texture locations... "));
				unsafe {
					let texture_coord_array = js_sys::Float32Array::view(&merged_array);
				
					context.buffer_data_with_array_buffer_view
					(
						WebGl2RenderingContext::ARRAY_BUFFER,
						&texture_coord_array,
						WebGl2RenderingContext::STATIC_DRAW
					);
				}
				rust_verbose(&("...buffering complete."));

				//Tell GPU how to extract vertex data from the buffer
				let position_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_position") as u32;
				context.vertex_attrib_pointer_with_i32
				(
					position_attribute_location, //index
					3, //size
					WebGl2RenderingContext::FLOAT, //data type
					false, //normalized
					20, //stride
					0 //offset
				);

				//Tell GPU how to extract texture data from the buffer
				let texture_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_texcoord") as u32;
				context.vertex_attrib_pointer_with_i32
				(
					texture_attribute_location, //index
					2, //size
					WebGl2RenderingContext::FLOAT, //data type
					false, //normalized 
					20, //stride
					12 //offset
				);

				context.enable_vertex_attrib_array(position_attribute_location);
				context.enable_vertex_attrib_array(texture_attribute_location);
				
				//Buffer the texture image
				rust_verbose(&("Starting to buffer texture image... "));
				let texture = context.create_texture().ok_or("failed to create texture")?;
				context.active_texture(WebGl2RenderingContext::TEXTURE0);
			    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
				let image = self.create_image_as_uint8_array()?;
				context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
				let _ = 
				match 
					context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_js_u8_array
					(
						WebGl2RenderingContext::TEXTURE_2D,
						0,
						WebGl2RenderingContext::RGBA8 as i32,
						self.texture_height,
						self.texture_width,
						//320,
						//320,
						0,
						WebGl2RenderingContext::RGBA, // format
						WebGl2RenderingContext::UNSIGNED_BYTE, // type
						Some(&image)
					)
				{
					Ok(result) => result,
					Err(_err) => panic!("failed to send image data to texture buffer.")
				};
				context.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
				rust_verbose(&("...texture buffering complete."));

			/*
			Manage Indices for model
			*/
			rust_verbose(&("Starting to buffer vertex & texture position indices... "));
			self.vertex_index_buffer = context.create_buffer();
			context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
			unsafe {
				let tmp_indices: Vec<u16> = (0..(merged_array.len() / 5) as u16).collect();
				self.indices_size = tmp_indices.len();
				let converted_indices = js_sys::Uint16Array::view(&tmp_indices);
				rust_super_verbose(&(converted_indices.to_string().as_string().unwrap()));
				context.buffer_data_with_array_buffer_view(
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
				self.vertex_buffer = context.create_buffer();
				context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_buffer.as_ref());

				let position_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_position") as u32;
				context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);	
						
				let vert_array = js_sys::Float32Array::view(&vertex_positions);
				context.buffer_data_with_array_buffer_view
				(
					WebGl2RenderingContext::ARRAY_BUFFER,
					&vert_array,
					WebGl2RenderingContext::STATIC_DRAW,
				);
				context.enable_vertex_attrib_array(position_attribute_location);
				rust_verbose(&("..Vertex data fully buffered."));
			}

			/*
				Manage Colors for model
			*/
			unsafe {
				rust_verbose(&("Starting to buffer color data... "));
				self.color_buffer = context.create_buffer();
				context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.color_buffer.as_ref());
				let color_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_color") as u32;
				context.vertex_attrib_pointer_with_i32(color_attribute_location, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
				context.enable_vertex_attrib_array(color_attribute_location);
		
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
				context.buffer_data_with_array_buffer_view
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
			self.vertex_index_buffer = context.create_buffer();
			context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
			unsafe {
				let tmp_indices = self.get_vertex_indices();
				self.indices_size = tmp_indices.len();	 
				let converted_indices = js_sys::Uint16Array::view(&tmp_indices);
				context.buffer_data_with_array_buffer_view(
					WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
					&converted_indices,
					WebGl2RenderingContext::STATIC_DRAW,
				);
			}
			rust_verbose(&("..indice buffering complete."));


		}
		return Ok(());
	}
}

mod object_loader;