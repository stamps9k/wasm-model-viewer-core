use wavefront_obj::obj::ObjSet;
use wavefront_obj::mtl::MtlSet;
use std::collections::HashMap;

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
				rust_info(&("Buffering model ".to_owned() + n.to_string().as_str() + ": " + &objset.objects[n].name + "to GPU..."));
				let mut tmp_obj: WebGl2WavefrontObject = WebGl2WavefrontObject::new(objset.objects[n].clone(), mtls.clone(), textures.clone())?;
				let _ = tmp_obj.buffer(&self.context, &self.program);
				self.update_l_and_s_values(&tmp_obj);
				self.objects.push(tmp_obj);
				rust_info(&"...model buffering complete.");
			}
		}

		self.set_projection();

		return Ok(());
	}

	// Checks the object's largest and smallest vertex positions and updates the frame if required. 
	fn update_l_and_s_values(&mut self, obj: &WebGl2WavefrontObject)
	{
		//Check and update x if required
		if (obj.largest[0]) > self.largest[0]
		{
			self.largest[0] = obj.largest[0] as f32;
		} 
		else if (obj.smallest[0] as f32) < self.smallest[0]
		{
			self.smallest[0] = obj.smallest[0] as f32;
		}

		//Check and update y if required
		if (obj.largest[1] as f32) > self.largest[1]
		{
			self.largest[1] = obj.largest[1] as f32;
		} 
		else if (obj.smallest[1] as f32) < self.smallest[1]
		{
			self.smallest[1] = obj.smallest[1] as f32;
		}

		//Check and update z if required
		if (obj.largest[2] as f32) > self.largest[2]
		{
			self.largest[2] = obj.largest[2] as f32;
		} 
		else if (obj.smallest[2] as f32) < self.smallest[2]
		{
			self.smallest[2] = obj.smallest[2] as f32;
		}
	}
}