use std::{ collections::HashMap };

use crate::logger::*;

use wavefront_obj::mtl::Material;
use wavefront_obj::obj::Primitive;

use super::WebGl2WavefrontObject;

impl WebGl2WavefrontObject
{
	/*
	 *
	 * Generates an merged array of vertex positions + RGBA color values for a diffuse object 
	 *
	 */
	pub(in super) fn get_merged_vertex_and_color_values(&mut self, vertex_positions: &Vec<f32>, vertex_indices: &Vec<u32>, color_array: &Vec<f32>) -> Vec<f32>
	{
		let mut merged_vertex_and_color_values: Vec<f32> = Vec::new();

		for n in 0..vertex_indices.len()
		{
			merged_vertex_and_color_values.push(vertex_positions[(vertex_indices[n] as usize * 3) + 0]);
			merged_vertex_and_color_values.push(vertex_positions[(vertex_indices[n] as usize * 3) + 1]);
			merged_vertex_and_color_values.push(vertex_positions[(vertex_indices[n] as usize * 3) + 2]);
			merged_vertex_and_color_values.push(color_array[(n * 4) + 0]);
			merged_vertex_and_color_values.push(color_array[(n * 4) + 1]);
			merged_vertex_and_color_values.push(color_array[(n * 4) + 2]);
			merged_vertex_and_color_values.push(color_array[(n * 4) + 3]);
		}

		self.log_merged_vertex_and_color_values(&merged_vertex_and_color_values);

		return merged_vertex_and_color_values;
	}

	/*
	 *
	 * Generate a list of 3 RGBA colors for each face in the model 
	 *
	 */
	pub(in super) fn get_color_values(&mut self) -> Vec<f32>
	{
		let mut colors_out: Vec<f32> = Vec::new();
		let mut mtls_lookup: HashMap<String, Material> = HashMap::new();

		for i in 0..self.mtls.as_ref().unwrap().materials.len()
		{
			mtls_lookup.insert(self.mtls.as_ref().unwrap().materials[i].name.clone(), self.mtls.as_ref().unwrap().materials[i].clone());
		}

		for i in 0..self.obj.geometry.len()
		{
			let mtl = mtls_lookup.get(&self.obj.geometry[i].material_name.as_ref().unwrap().clone()).unwrap();

			for _j in 0..self.obj.geometry[i].shapes.len()
			{
				colors_out.push(mtl.color_diffuse.r as f32);
				colors_out.push(mtl.color_diffuse.g as f32);
				colors_out.push(mtl.color_diffuse.b as f32);
				colors_out.push(1.0); // For now assumes all materials have alpha of 1.0
				colors_out.push(mtl.color_diffuse.r as f32);
				colors_out.push(mtl.color_diffuse.g as f32);
				colors_out.push(mtl.color_diffuse.b as f32);
				colors_out.push(1.0); // For now assumes all materials have alpha of 1.0
				colors_out.push(mtl.color_diffuse.r as f32);
				colors_out.push(mtl.color_diffuse.g as f32);
				colors_out.push(mtl.color_diffuse.b as f32);
				colors_out.push(1.0); // For now assumes all materials have alpha of 1.0
			}
		}
		return colors_out;

	}

	/*
	*
	*	Fetch the vertex index data as stored in the model. 
	*	Note 1 - indexes are reduced by 1 versus the raw model data due to OpenGL using 0 indexing
	*	Note 2 - fetched in the order y, z, x as for some reason the parser library stores the data in a different order to the model
	*
	*/
	pub(in super) fn get_vertex_indices(&mut self) -> Vec<u32> 
	{
		let mut shapes_out: Vec<u32> = Vec::new();

		for i in 0..self.obj.geometry.len()
		{
			for j in 0..self.obj.geometry[i].shapes.len()
			{
				let Primitive::Triangle(x, y, z) = self.obj.geometry[i].shapes[j].primitive else { rust_log(&"Not Triangle", &"warn_wasm_parse"); continue; };
				shapes_out.push(y.0 as u32);
				shapes_out.push(z.0 as u32);
				shapes_out.push(x.0 as u32);
			}
		}

		self.log_vertex_indices(&shapes_out);	

		return shapes_out;
	}

	/*
	*
	*	Fetch the vertex position data as stored in the model. 
	*
	*/
	pub(in super) fn get_vertex_positions(&mut self) -> Vec<f32> 
	{
		let mut vertices_out: Vec<f32> = Vec::new();

		// Reset scaling values
		self.largest = [0.0, 0.0, 0.0];
		self.smallest = [0.0, 0.0, 0.0];

		for n in 0..self.obj.vertices.len()
		{
			vertices_out.push(self.obj.vertices[n].x as f32);
			vertices_out.push(self.obj.vertices[n].y as f32);
			vertices_out.push(self.obj.vertices[n].z as f32);

			//Check and update x if required
            if (self.obj.vertices[n].x as f32) > self.largest[0]
            {
                self.largest[0] = self.obj.vertices[n].x as f32;
            } 
            else if (self.obj.vertices[n].x as f32) < self.smallest[0]
            {
                self.smallest[0] = self.obj.vertices[n].x as f32;
            }

            //Check and update y if required
            if (self.obj.vertices[n].y as f32) > self.largest[1]
            {
                self.largest[1] = self.obj.vertices[n].y as f32;
            } 
            else if (self.obj.vertices[n].y as f32) < self.smallest[1]
            {
                self.smallest[1] = self.obj.vertices[n].y as f32;
            }

            //Check and update z if required
            if (self.obj.vertices[n].z as f32) > self.largest[2]
            {
                self.largest[2] = self.obj.vertices[n].z as f32;
            } 
            else if (self.obj.vertices[n].z as f32) < self.smallest[2]
            {
                self.smallest[2] = self.obj.vertices[n].z as f32;
            }

		}

		return vertices_out;
	}

	/*
	*
	* Fetch the texture position data as stored in the model. 
	*
	*/
	pub(in super) fn get_texture_positions(&mut self) -> Vec<f32> 
	{
		let mut vertices_out: Vec<f32> = Vec::new();

		for n in 0..self.obj.tex_vertices.len()
		{
			vertices_out.push(self.obj.tex_vertices[n].u as f32);
			vertices_out.push(self.obj.tex_vertices[n].v as f32);
		}
		return vertices_out;
	}

	/*
	*
	*	Fetch the texture index data as stored in the model. 
	*	Note 1 - fetched in the order y, z, x as for some reason the parser library stores the data in a different order to the model
	*
	*/
	pub(in super) fn get_texture_indices(&mut self) -> Vec<u32>
	{
		let mut shapes_out: Vec<u32> = Vec::new();

		for i in 0..self.obj.geometry.len()
		{
			for j in 0..self.obj.geometry[i].shapes.len()
			{
				let Primitive::Triangle(x, y, z) = self.obj.geometry[i].shapes[j].primitive else { break };
				match y.1 
				{
					Some(y) => shapes_out.push(y as u32),
					None => break
				};
				match z.1 
				{
					Some(z) => shapes_out.push(z as u32),
					None => break
				};
				match x.1 
				{
					Some(x) => shapes_out.push(x as u32),
					None => break
				};
			}
		}

		return shapes_out;
	}

	/*
	*
	*	Merge the vertex and texture position data so that it can be stored in a single OpenGL buffer
	*
	*/
	pub(in super) fn merge_vertex_and_texture_positions(&mut self, vertex_positions: &Vec<f32>, vertex_indices: &Vec<u32>, texture_positions: &Vec<f32>, texture_indices: &Vec<u32>) -> Vec<f32>
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

		self.log_merged_vertex_and_texture_positions(&merged_vertex_and_texture_positions);

		return merged_vertex_and_texture_positions;
	}

	/*
	*
	*	Merge the vertex, texture and color data so that it can be stored in a single OpenGL buffer
	*
	*/
	pub(in super) fn merge_vertex_texture_and_color_data(&mut self, vertex_positions: &Vec<f32>, vertex_indices: &Vec<u32>, texture_positions: &Vec<f32>, texture_indices: &Vec<u32>, color_array: &Vec<f32>) -> Vec<f32>
	{
		let mut merged_vertex_texture_and_color_data: Vec<f32> = Vec::new();

		for n in 0..vertex_indices.len()
		{
			merged_vertex_texture_and_color_data.push(vertex_positions[(vertex_indices[n] as usize * 3) + 0]);
			merged_vertex_texture_and_color_data.push(vertex_positions[(vertex_indices[n] as usize * 3) + 1]);
			merged_vertex_texture_and_color_data.push(vertex_positions[(vertex_indices[n] as usize * 3) + 2]);
			merged_vertex_texture_and_color_data.push(texture_positions[(texture_indices[n] as usize * 2) + 0]);
			merged_vertex_texture_and_color_data.push(1.0 - (texture_positions[(texture_indices[n] as usize * 2) + 1]));
			merged_vertex_texture_and_color_data.push(color_array[(n * 4) + 0]);
			merged_vertex_texture_and_color_data.push(color_array[(n * 4) + 1]);
			merged_vertex_texture_and_color_data.push(color_array[(n * 4) + 2]);
			merged_vertex_texture_and_color_data.push(color_array[(n * 4) + 3]);
		}

		self.log_merged_vertex_texture_and_color_data(&merged_vertex_texture_and_color_data);

		return merged_vertex_texture_and_color_data;
	}

	/*
	*
	*	Log the vertex indices in a nice format 
	*
	*/
	pub(in super) fn log_vertex_indices(&mut self, vertex_indices: &Vec<u32>)
	{
		rust_log(&"Loaded vertex indices are:", &"super_super_verbose_wasm_parse");
		for n in 0..vertex_indices.len()
		{
			if n % 3 == 0
			{
				rust_log
				(
					&format!("{}, {}, {}", vertex_indices[n], vertex_indices[n + 1], vertex_indices[n + 2]), 
					&"super_super_verbose_wasm_parse"
				);
			}
		}
	}

	/*
	*
	*	Log the merged vertex and texture coordinates in a nice format
	*
	*/
	pub(in super) fn log_merged_vertex_and_color_values(&mut self, coords: &Vec<f32>)
	{
		rust_log(&"Merged vertex & color values array is:", &"super_super_verbose_wasm_parse");
		for n in 0..coords.len()
		{
			if n % 7 == 0
			{
				rust_log
				(
					&format!
					(
						"{}, {}, {} - {}, {}, {}, {}", 
						coords[n], coords[n + 1], coords[n + 2], coords[n + 3], coords[n + 4], coords[n + 5], coords[n + 6]
					), 
					&"super_super_verbose_wasm_parse"
				);
			}
		}
	}

	/*
	*
	*	Log the merged vertex and texture coordinates in a nice format
	*
	*/
	pub(in super) fn log_merged_vertex_and_texture_positions(&mut self, coords: &Vec<f32>)
	{
		rust_log(&"Merged vertex & texture positions array is:", &"super_super_verbose_wasm_parse");
		for n in 0..coords.len()
		{
			if n % 5 == 0
			{
				rust_log
				(
					&format!
					(
						"{}, {}, {} - {}, {}", 
						coords[n], coords[n + 1], coords[n + 2], coords[n + 3], coords[n + 4]
					), 
					&"super_super_verbose_wasm_parse"
				);
			}
		}
	}

	/*
	*
	*	Log the merged vertex, texture and color data in a nice format
	*
	*/
	pub(in super) fn log_merged_vertex_texture_and_color_data(&mut self, coords: &Vec<f32>)
	{
		rust_log(&"Merged vertex, texture & color array is:", &"super_super_verbose_wasm_parse");
		for n in 0..coords.len()
		{
			if n % 9 == 0
			{
				rust_log
				(
					&format!
					(
						"{}, {}, {} - {}, {} - {}, {}, {}, {}", 
						coords[n], coords[n + 1], coords[n + 2], coords[n + 3], coords[n + 4], coords[n + 5], coords[n + 6], coords[n + 7], coords[n + 8]
					), 
					&"super_super_verbose_wasm_parse"
				);
			}
		}
	}

}