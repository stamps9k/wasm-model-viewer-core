use rand::Rng;
use web_sys::WebGl2RenderingContext;
use wavefront_obj::obj::Object;
use wavefront_obj::obj::ObjSet;

use crate::logger::*;
use crate::object_loader;

use super::WebGl2Frame;

impl WebGl2Frame
{
    pub(in super) fn buffer_scene(&mut self, objset: &ObjSet, textures: &Vec<String>) -> Result<(), String>
	{
		//rust_info(&"Loading textures to memory...");
		rust_super_verbose(&("Texture is: ".to_owned() + &textures[0]));
		//rust_info(&"... texture loading complete.");

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

    pub(in super) fn buffer_obj(&mut self, obj: &Object, texture_b64: String) -> Result<(), String>
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

		if texture_b64 != String::from("bad_value")
		{
			rust_verbose(&("Object: ".to_owned() + obj.name.as_str() + "identified as textured model. Processing accordingly"));
			
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

				//This breaks on donut upload
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
}