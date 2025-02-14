use crate::logger;
use wavefront_obj::obj::*;
use js_sys::*;
use std::io::Cursor;
use image::ImageReader;

/*
*
*	Fetch the vertex index data as stored in the model. 
*	Note 1 - indexes are reduced by 1 versus the raw model data due to OpenGL using 0 indexing
*	Note 2 - fetched in the order y, z, x as for some reason the parser library stores the data in a different order to the model
*
*/
pub fn get_vertex_indices(data: &wavefront_obj::obj::Object) -> Vec<u16> 
{
	let mut shapes_out: Vec<u16> = Vec::new();

	for n in 0..data.geometry[0].shapes.len()
	{
		let Primitive::Triangle(x, y, z) = data.geometry[0].shapes[n].primitive else { break };
		shapes_out.push(y.0 as u16);
		shapes_out.push(z.0 as u16);
		shapes_out.push(x.0 as u16);
	}

	log_vertex_indices(&shapes_out);	

	return shapes_out;
}

/*
*
*	Fetch the vertex position data as stored in the model. 
*
*/
pub fn get_vertex_positions(data: &wavefront_obj::obj::Object) -> Vec<f32> 
{
	let mut vertices_out: Vec<f32> = Vec::new();

	for n in 0..data.vertices.len()
	{
		vertices_out.push(data.vertices[n].x as f32);
		vertices_out.push(data.vertices[n].y as f32);
		vertices_out.push(data.vertices[n].z as f32);
		
	}

	return vertices_out;
}

/*
*
* Fetch the texture position data as stored in the model. 
*
*/
pub fn get_texture_positions(data: &wavefront_obj::obj::Object) -> Vec<f32> 
{
	let mut vertices_out: Vec<f32> = Vec::new();

	for n in 0..data.tex_vertices.len()
	{
		vertices_out.push(data.tex_vertices[n].u as f32);
		vertices_out.push(data.tex_vertices[n].v as f32);
		//vertices_out.push(data.tex_vertices[n].w as f32);
	}
	return vertices_out;
}

/*
*
*	Fetch the texture index data as stored in the model. 
*	Note 1 - fetched in the order y, z, x as for some reason the parser library stores the data in a different order to the model
*
*/
pub fn get_texture_indices(data: &wavefront_obj::obj::Object) -> Vec<u16>
{
	let mut shapes_out: Vec<u16> = Vec::new();

	for n in 0..data.geometry[0].shapes.len()
	{
		let Primitive::Triangle(x, y, z) = data.geometry[0].shapes[n].primitive else { break };
		match y.1 
		{
			Some(y) => shapes_out.push(y as u16),
			None => break
		};
		match z.1 
		{
			Some(z) => shapes_out.push(z as u16),
			None => break
		};
		match x.1 
		{
			Some(x) => shapes_out.push(x as u16),
			None => break
		};
	}

	return shapes_out;
}

/*
*
*	Merge the vertex and texture position data so that it can be stored in a single OpenGL buffer
*
*/
pub fn merge_vertex_and_texture_positions(vertex_positions: &Vec<f32>, vertex_indices: &Vec<u16>, texture_positions: &Vec<f32>, texture_indices: &Vec<u16>) -> Vec<f32>
{
	let mut merged_vertex_and_texture_positions: Vec<f32> = Vec::new();

	for n in 0..vertex_indices.len()
	{
		merged_vertex_and_texture_positions.push(vertex_positions[(vertex_indices[n] as usize * 3) + 0]);
		merged_vertex_and_texture_positions.push(vertex_positions[(vertex_indices[n] as usize * 3) + 1]);
		merged_vertex_and_texture_positions.push(vertex_positions[(vertex_indices[n] as usize * 3) + 2]);
		merged_vertex_and_texture_positions.push(texture_positions[(texture_indices[n] as usize * 2) + 0]);
		merged_vertex_and_texture_positions.push(1.0 - (texture_positions[(texture_indices[n] as usize * 2) + 1]));
	}

	log_merged_vertex_and_texture_positions(&merged_vertex_and_texture_positions);

	return merged_vertex_and_texture_positions;
}

/*
*
*	Convert the passed base64 image data into a raw u8 array of rbga values. 
*
*/
pub fn create_image_as_uint8_array(base64_png: &str) -> Result<Uint8Array, String> 
{
	// Convert base64 to a binary array
	let bytes = base64::decode(base64_png).map_err(|_| "Failed to decode base64")?;

	let img1 = match ImageReader::new(Cursor::new(bytes)).with_guessed_format()
	{
		Ok(img1) => img1,
		Err(e) => return Err(e.to_string())
	};

	let img2 = match img1.decode()
	{
		Ok(img2) => img2,
		Err(e) => return Err(e.to_string())
	};

	let rgba_img = img2.to_rgba8();

	// Get image dimensions
    let (width, height) = rgba_img.dimensions();
	web_sys::console::log_1(&("Image Size: ".to_owned() + width.to_string().as_str() + " x " + height.to_string().as_str()).into());

	// Access raw pixel data
    let pixels = rgba_img.as_raw();

	// Create a Blob from binary data
    let array = js_sys::Uint8Array::from(pixels.as_slice());

	log_js_uint8_array(&array);

	return Ok(array);	
}

/*
*
*	Log the vertex indices in a nice format 
*
*/
pub fn log_vertex_indices(vertex_indices: &Vec<u16>)
{
	logger::rust_super_verbose(&("Loaded vertex indices are: "));
	for n in 0..vertex_indices.len()
	{
		if n % 3 == 0
		{
			logger::rust_super_verbose
			(
				&(
					vertex_indices[n].to_string().as_str().to_owned() + 
					" " + 
					vertex_indices[n + 1].to_string().as_str() +
					" " + 
					vertex_indices[n + 2].to_string().as_str()	
				)
			);
		}
	}
}

/*
*
*	Log the merged vertex and texture coordinates in a nice format
*
*/
pub fn log_merged_vertex_and_texture_positions(coords: &Vec<f32>)
{
	logger::rust_verbose
	(
		&(
			"Merged vertex & texture positions size is ".to_owned() + coords.len().to_string().as_str() +
			" covering " + (coords.len() / 5).to_string().as_str() + " items"
		)
	);
	logger::rust_super_verbose(&("Merged vertex & texture positions buffer is : "));
	for n in 0..coords.len()
	{
		if n % 5 == 0
		{
			logger::rust_super_verbose
			(
				&(
					coords[n].to_string().as_str().to_owned() + ", " + coords[n + 1].to_string().as_str() + ", " + coords[n + 2].to_string().as_str() + 
					" - " + 
					coords[n + 3].to_string().as_str() + " " + coords[n + 4].to_string().as_str()
				)
			);
		}
	}
}

/*
*
*	Log the RGBA elements in image array
*
*/
pub fn log_js_uint8_array(array: &js_sys::Uint8Array)
{
	logger::rust_super_verbose(&("Loaded texure coordinates are: "));
	logger::rust_super_verbose(&(array.to_string().as_string().unwrap()));
}
