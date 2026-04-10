use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use web_sys::WebGlBuffer;
use std::collections::HashMap;


use crate::logger::*;

pub struct WebGl2WavefrontObject
{
	pub marked_for_deletion: bool,
	obj: wavefront_obj::obj::Object,
	mtls: Option<wavefront_obj::mtl::MtlSet>,
	pub vertex_buffer: Option<WebGlBuffer>,
	pub vertex_and_color_buffer: Option<WebGlBuffer>,
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
			marked_for_deletion: false,
			obj: obj,
			mtls: mtls,
			vertex_buffer: None,
			vertex_and_color_buffer: None,
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
		 *	Process the object through different route depending on contents present (or not)
		 *
		 */
		if self.is_textured_model() // Object is a textured model
		{
			rust_log(&format!("Object: {} identified as textured model. Processing accordingly", self.obj.name), &"info_wasm_parse");			
			self.textured_object_gpu_buffer(context, program)?;
		} else if self.is_diffuse_model() {
			rust_log(&format!("Object: {} identified as a diffuse model. Processing accordingly", self.obj.name), &"info_wasm_parse");
			self.diffuse_object_gpu_buffer(context, program)?;
		} else if self.is_unlit_model() { // No textures and no materials defined 
			rust_log(&format!("Object: {} identified as an unlit model. Processing accordingly", self.obj.name), &"info_wasm_parse");
			self.unlit_object_gpu_buffer(context, program)?;
		} else {
			rust_log(&format!("Object: {} did not match any known type. Processing as an unlit model.", self.obj.name), &"warn_wasm_parse");
			self.unlit_object_gpu_buffer(context, program)?;
		}

		return Ok(());
	}

	pub(in super) fn is_textured_model(&self) -> bool
    {
        return 
            self.mtls != None &&
            (
                self.mtls.as_ref() .unwrap().materials[0].ambient_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].diffuse_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].specular_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].specular_exponent_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].dissolve_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].displacement_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].decal_map != None ||
                self.mtls.as_ref() .unwrap().materials[0].bump_map != None
            )
    }

	pub(in super) fn is_diffuse_model(&self) -> bool
    {
        return 
            self.mtls != None &&
            (
                self.mtls.as_ref() .unwrap().materials[0].ambient_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].diffuse_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].specular_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].specular_exponent_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].dissolve_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].displacement_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].decal_map == None &&
                self.mtls.as_ref() .unwrap().materials[0].bump_map == None
            )
    }

	pub(in super) fn is_unlit_model(&self) -> bool
	{
		return self.mtls == None;
	}

	pub fn cleanup(&mut self, context: &WebGl2RenderingContext)
	{
		context.disable_vertex_attrib_array(0); // a_position
    	context.disable_vertex_attrib_array(1);
		context.delete_buffer(self.vertex_buffer.as_ref());
		context.delete_buffer(self.vertex_and_texture_buffer.as_ref());
		context.delete_buffer(self.vertex_index_buffer.as_ref());
		context.delete_buffer(self.color_buffer.as_ref());
	}
}

mod mesh_data_extractor;
mod gpu_drawer;
mod gpu_uploader;
mod texture_decoder;