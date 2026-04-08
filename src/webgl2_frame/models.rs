use wavefront_obj::obj::ObjSet;
use wavefront_obj::mtl::MtlSet;
use std::collections::HashMap;
use webgl_matrix::*;

use crate::matrix_helper::scaling_matrix;
use crate::logger::*;
use crate::webgl2_wavefront_object::WebGl2WavefrontObject;

use super::WebGl2Frame;

impl WebGl2Frame
{
    pub(in super) fn buffer_scene(&mut self, objset: &ObjSet, mtls: &Option<MtlSet>, textures: &Option<HashMap<String, String>>) -> Result<(), String>
	{
		for n in 0..(&objset).objects.len()
		{
			//Ignore junk objects
			if (&objset).objects[n].vertices.len() != 0	
			{
				rust_log(&format!("Buffering model {} to GPU...", &objset.objects[n].name), &"verbose_wasm_gpu_data.");
				let mut tmp_obj: WebGl2WavefrontObject = WebGl2WavefrontObject::new(objset.objects[n].clone(), mtls.clone(), textures.clone())?;
				let _ = tmp_obj.buffer(&self.context, &self.program);
				rust_log(&format!("Setting largest and smallest values for model {}...", &objset.objects[n].name), &"verbose_wasm_parse.");
				self.update_l_and_s_values(&tmp_obj);
				rust_log(&"...largest and smallest values set successfully", &"verbose_wasm_parse.");
				self.objects.push(tmp_obj);
				rust_log(&format!("model {} buffering complete...", &objset.objects[n].name), &"verbose_wasm_gpu_data.");
			}
		}

		self.init_model_normalize_matrix();
		self.set_projection();

		return Ok(());
	}

	/*
		Set the model matrix so that the scene is centered and scaled consistently across different objects
	*/
	fn init_model_normalize_matrix(&mut self)
	{
		rust_log(&"Generating the model matrix...", &"verbose_wasm_math");
		self.model_normalize_matrix = Mat4::identity(); //Create the translation matrix to centralise object ontop of camera
		self.model_normalize_matrix = *self.model_normalize_matrix.translate(&self.get_centralisation()); //Create the translation matrix to centralise object ontop of camera
		m4_pretty_print_super_verbose(&"Model matrix after centralising:", &self.model_normalize_matrix);

		let scale_mat: Mat4 = scaling_matrix(self.get_scaling()); //Create the scaling matrix
		self.model_normalize_matrix.mul(&scale_mat); //Combine so that scaled model moves the right amount to sit on camera
		m4_pretty_print_super_verbose(&"Model matrix after scaling:", &self.model_normalize_matrix);
		
		let position_index = self.context.get_uniform_location(self.program.as_ref().unwrap(), "u_model_normalize_matrix");
		self.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &self.model_normalize_matrix);
		rust_log(&"...Model matrix successfully generated.", &"verbose_wasm_math");
	}

	// Checks the object's largest and smallest vertex positions and updates the frame if required. 
	fn update_l_and_s_values(&mut self, obj: &WebGl2WavefrontObject)
	{
		//Check and update x if required
		if (obj.largest[0]) > self.largest[0]
		{
			self.largest[0] = obj.largest[0] as f32;
		} 
		if (obj.smallest[0] as f32) < self.smallest[0]
		{
			self.smallest[0] = obj.smallest[0] as f32;
		}

		//Check and update y if required
		if (obj.largest[1] as f32) > self.largest[1]
		{
			self.largest[1] = obj.largest[1] as f32;
		} 
		if (obj.smallest[1] as f32) < self.smallest[1]
		{
			self.smallest[1] = obj.smallest[1] as f32;
		}

		//Check and update z if required
		if (obj.largest[2] as f32) > self.largest[2]
		{
			self.largest[2] = obj.largest[2] as f32;
		} 
		if (obj.smallest[2] as f32) < self.smallest[2]
		{
			self.smallest[2] = obj.smallest[2] as f32;
		}
		rust_log
		(
			&format!
			(
				r#"Largest and smallest values are:
				Largest:  {}, {}, {}
				Smallest: {}, {}, {}"#,
				self.largest[0], self.largest[1], self.largest[2],
				self.smallest[0], self.smallest[1], self.smallest[2], 
			), 
			&"super_verbose_wasm_parse"
		);

	}
}