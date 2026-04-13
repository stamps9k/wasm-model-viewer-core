use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use rand::Rng;

use crate::logger::*;

use super::WebGl2WavefrontObject;

impl WebGl2WavefrontObject
{
    pub fn draw(&self, context: &WebGl2RenderingContext, program: &WebGlProgram)
    {
        if self.is_unlit_model()
        {
            self.draw_unlit_object(context, program);
        } else if self.is_diffuse_model()
        {
            self.draw_diffuse_object(context, program);
        } else if self.is_textured_model()
        {
            self.draw_textured_object(context, program);
        }

    }

    pub(in super) fn draw_unlit_object(&self, context: &WebGl2RenderingContext, program: &WebGlProgram)
    {
        rust_log("UNLIT OBJECT. Binding position data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_buffer.as_ref());
        let position_attribute_location = context.get_attrib_location(program, "a_position") as u32;
        context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(position_attribute_location);
        rust_log("...position data binding complete.", &"super_super_verbose_gpu_data");

        rust_log("UNLIT OBJECT. Binding color data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.color_buffer.as_ref());
        let color_attribute_location = context.get_attrib_location(program, "a_color") as u32;
        context.vertex_attrib_pointer_with_i32(color_attribute_location, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(color_attribute_location);
        rust_log("...color data binding complete.", &"super_super_verbose_gpu_data");

        rust_log("UNLIT OBJECT. Binding index data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        rust_log("...index data binding complete.", &"super_super_verbose_gpu_data");
        
        context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, self.indices_size as i32, WebGl2RenderingContext::UNSIGNED_INT, 0.0);
        rust_log(&format!("...{} indices drawn.", self.indices_size), &"super_super_verbose_wasm_gpu_data");
    }

    pub(in super) fn  draw_diffuse_object(&self, context: &WebGl2RenderingContext, program: &WebGlProgram)
    {
        rust_log("DIFFUSE OBJECT. Binding position and color data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_and_color_buffer.as_ref());
        let position_attribute_location = context.get_attrib_location(program, "a_position") as u32; 
        context.vertex_attrib_pointer_with_i32
        (
            position_attribute_location, 
            3, 
            WebGl2RenderingContext::FLOAT, 
            false, 
            28, 
            0
        );
        context.enable_vertex_attrib_array(position_attribute_location);
        let color_attribute_location = context.get_attrib_location(program, "a_color") as u32;
        context.vertex_attrib_pointer_with_i32
        (
            color_attribute_location, 
            4, 
            WebGl2RenderingContext::FLOAT, 
            false, 
            28, 
            12
        );
        context.enable_vertex_attrib_array(color_attribute_location);

        rust_log("...position and color data binding complete.", &"super_super_verbose_gpu_data");

        rust_log("UNLIT OBJECT. Binding index data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        rust_log("...index data binding complete.", &"super_super_verbose_gpu_data");

        context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, self.indices_size as i32, WebGl2RenderingContext::UNSIGNED_INT, 0.0);
        rust_log(&format!("...{} indices drawn.", self.indices_size), &"super_super_verbose_wasm_gpu_data");
    }

    pub(in super) fn draw_textured_object(&self, context: &WebGl2RenderingContext, program: &WebGlProgram)
    {
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_and_texture_buffer.as_ref());				
				
        rust_log("TEXTURED OBJECT. Binding position data...", &"super_super_verbose_gpu_data");
        let position_attribute_location = context.get_attrib_location(program, "a_position") as u32;
        context.vertex_attrib_pointer_with_i32
        (
            position_attribute_location, //index
            3, //size
            WebGl2RenderingContext::FLOAT, //data type
            false, //normalized
            20, //stride
            0 //offset
        );
        rust_log("...position data binding complete.", &"super_super_verbose_gpu_data");

        rust_log("TEXTURED OBJECT. Binding texture data...", &"super_super_verbose_gpu_data");				
        let texture_attribute_location = context.get_attrib_location(program, "a_texcoord") as u32;
        context.vertex_attrib_pointer_with_i32
        (
            texture_attribute_location, //index
            2, //size
            WebGl2RenderingContext::FLOAT, //data type
            false, //normalized 
            20, //stride
            12 //offset
        );
        rust_log("...texture data binding complete.", &"super_super_verbose_gpu_data");

        context.enable_vertex_attrib_array(position_attribute_location);
        context.enable_vertex_attrib_array(texture_attribute_location);

        rust_log("TEXTURED OBJECT. Binding index data...", &"super_super_verbose_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        rust_log("...index data binding complete.", &"super_super_verbose_gpu_data");

        context.draw_elements_with_f64(WebGl2RenderingContext::TRIANGLES, self.indices_size as i32, WebGl2RenderingContext::UNSIGNED_INT, 0.0);
        rust_log(&format!("...{} indices drawn.", self.indices_size), &"super_super_verbose_wasm_gpu_data");

    }
}