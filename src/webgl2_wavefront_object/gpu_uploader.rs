use web_sys::WebGl2RenderingContext;
use web_sys::WebGlProgram;
use rand::Rng;

use crate::logger::*;

use super::WebGl2WavefrontObject;

impl WebGl2WavefrontObject
{
    pub(in super) fn textured_object_gpu_buffer(&mut self, context: &WebGl2RenderingContext, program: &Option<WebGlProgram>) -> Result<(), String>
    {
        /*
            get the information from the object in a workable format
        */
        let vertex_positions: Vec<f32> = self.get_vertex_positions();
		rust_log(&format!("Vertices array is size: {}", vertex_positions.len()), &"verbose_wasm_parse");
		let vertex_indices: Vec<u32> = self.get_vertex_indices();
		rust_log(&format!("Vertex indices array is size: {}", vertex_indices.len()), &"verbose_wasm_parse");

		let texture_vertices: Vec<f32> = self.get_texture_positions();
		rust_log(&format!("Texture Vertices array is size: {}", texture_vertices.len()), &"verbose_wasm_parse");

		let texture_indices: Vec<u32> = self.get_texture_indices();
		rust_log(&format!("Texture Indices array is size: {}", texture_indices.len()), &"verbose_wasm_parse");

        /*
            Manage model texture and vertices
        */
        // First generate the texture and vertex info
        rust_log("Generating a list that has all unique combined vertex + texture positions...", &"info_wasm_parse");
        let merged_array: Vec<f32> = self.merge_vertex_and_texture_positions(&vertex_positions, &vertex_indices, &texture_vertices, &texture_indices);
        rust_log(&format!("Merged vertex & texture positions size is: {}", merged_array.len()), &"verbose_wasm_parse");
        rust_log("...combined vertex + texture positions list completed.", &"info_wasm_parse");

        // create the GPU buffer
        rust_log(&"Creating GPU buffer for vertex and texture positions array...", &"verbose_wasm_gpu_mem");
        self.vertex_and_texture_buffer = context.create_buffer();
        rust_log(&"...GPU buffer creation complete.", &"verbose_wasm_gpu_mem");
        rust_log(&"Binding data to GPU buffer for vertex and texture positions array...", &"verbose_wasm_gpu_data");
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_and_texture_buffer.as_ref());
        rust_log(&"Data successfully bound to GPU buffer.", &"verbose_wasm_gpu_data");

        //Put values into buffer
        rust_log(&"Starting to buffer vertex & texture locations...", &"verbose_wasm_gpu_data");
        unsafe 
        {
            let texture_coord_array = js_sys::Float32Array::view(&merged_array);
            context.buffer_data_with_array_buffer_view
            (
                WebGl2RenderingContext::ARRAY_BUFFER,
                &texture_coord_array,
                WebGl2RenderingContext::STATIC_DRAW
            );
        }
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");

        //Tell GPU how to extract vertex data from the buffer
        rust_log(&"Binding position attribute...", &"verbose_wasm_gpu_data");
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
        context.enable_vertex_attrib_array(position_attribute_location);
        rust_log(&"...binding complete.", &"verbose_wasm_gpu_data");

        //Tell GPU how to extract texture data from the buffer
        rust_log(&"Binding texcoord attribute...", &"verbose_wasm_gpu_data");
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
        context.enable_vertex_attrib_array(texture_attribute_location);
        rust_log(&"...binding complete.", &"verbose_wasm_gpu_data");
        
        //Buffer the texture image
        rust_log(&"Starting to buffer texture image...", &"verbose_wasm_gpu_data");
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
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");

        /*
            Manage Indices for model
        */
        rust_log(&"Starting to buffer vertex & texture position indices...", &"verbose_wasm_gpu_data");
        self.vertex_index_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        let tmp_indices: Vec<u32> = (0..(merged_array.len() / 5) as u32).collect();
        self.indices_size = tmp_indices.len();
        unsafe 
        {
            let converted_indices = js_sys::Uint32Array::view(&tmp_indices);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &converted_indices,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");

        return Ok(());
    }

    pub(in super) fn unlit_object_gpu_buffer(&mut self, context: &WebGl2RenderingContext, program: &Option<WebGlProgram>) -> Result<(), String>
    {
        /*
            get the information from the object in a workable format
        */
        let vertex_positions: Vec<f32> = self.get_vertex_positions();
		rust_log(&format!("Vertices array is size: {}", vertex_positions.len()), &"verbose_wasm_parse");
		let vertex_indices: Vec<u32> = self.get_vertex_indices();
		rust_log(&format!("Vertex indices array is size: {}", vertex_indices.len()), &"verbose_wasm_parse");

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
        rust_log(&"Starting to buffer vertex locations...", &"verbose_wasm_gpu_data");
        self.vertex_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_buffer.as_ref());

        let position_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_position") as u32;
        context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);	

        unsafe 
        {      
            let vert_array = js_sys::Float32Array::view(&vertex_positions);
            context.buffer_data_with_array_buffer_view
            (
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
            context.enable_vertex_attrib_array(position_attribute_location);
            rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");
        }

        /*
            Manage Colors for model
        */
        rust_log(&"Starting to buffer color data...", &"verbose_wasm_gpu_data");
        self.color_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.color_buffer.as_ref());
        let color_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_color") as u32;
        context.vertex_attrib_pointer_with_i32(color_attribute_location, 4, WebGl2RenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(color_attribute_location);

        //Currently junk colors. Only care about matching vertex count in sample cube
        let mut rng = rand::rng();
        let mut colors: Vec<f32> = Vec::new();
        for _ in 0..self.obj.vertices.len()
        {
            colors.push(rng.random_range(0.0..=1.0));
            colors.push(rng.random_range(0.0..=1.0));
            colors.push(rng.random_range(0.0..=1.0));
            colors.push(1.0);
        }

        unsafe 
        {
            let color_array = js_sys::Float32Array::view(&colors);
            context.buffer_data_with_array_buffer_view
            (
                WebGl2RenderingContext::ARRAY_BUFFER,
                &color_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");


        /*
            Manage Indices for model
        */
        rust_log(&"Starting to buffer vertex indices...", &"verbose_wasm_gpu_data");
        self.vertex_index_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        let tmp_indices = self.get_vertex_indices();
        self.indices_size = tmp_indices.len();	 
        unsafe 
        {
            let converted_indices = js_sys::Uint32Array::view(&tmp_indices);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &converted_indices,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");
        
        return Ok(());
    }

    pub(in super) fn diffuse_object_gpu_buffer(&mut self, context: &WebGl2RenderingContext, program: &Option<WebGlProgram>) -> Result<(), String>
    {
        /*
            get the information from the object in a workable format
        */
        let vertex_positions: Vec<f32> = self.get_vertex_positions();
		rust_log(&format!("Vertices array is size: {}", vertex_positions.len()), &"verbose_wasm_parse");
		let vertex_indices: Vec<u32> = self.get_vertex_indices();
		rust_log(&format!("Vertex indices array is size: {}", vertex_indices.len()), &"verbose_wasm_parse");
        let color_values: Vec<f32> = self.get_color_values();
        rust_log(&format!("Color array is size: {}", color_values.len()), &"verbose_wasm_parse");
        let merge_vertex_and_color_data: Vec<f32> = self.get_merged_vertex_and_color_values(&vertex_positions, &vertex_indices, &color_values);
        rust_log(&format!("Merged color and vertex array is size: {}", merge_vertex_and_color_data.len()), &"verbose_wasm_parse");

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
        rust_log(&"Starting to buffer vertex locations...", &"verbose_wasm_gpu_data");
        self.vertex_and_color_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vertex_and_color_buffer.as_ref());

        let position_attribute_location = context.get_attrib_location(program.as_ref().unwrap(), "a_position") as u32;
        context.vertex_attrib_pointer_with_i32
        (
            position_attribute_location, 
            3, 
            WebGl2RenderingContext::FLOAT, 
            false, 
            28, 
            0
        );	

        let color_attribute_location: u32 = context.get_attrib_location(program.as_ref().unwrap(), "a_color") as u32;
        context.vertex_attrib_pointer_with_i32
        (
            color_attribute_location, 
            4, 
            WebGl2RenderingContext::FLOAT, 
            false, 
            28, 
            12
        );	
        unsafe 
        {
            let vert_and_color_array = js_sys::Float32Array::view(&merge_vertex_and_color_data);
            context.buffer_data_with_array_buffer_view
            (
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_and_color_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        context.enable_vertex_attrib_array(position_attribute_location);
        context.enable_vertex_attrib_array(color_attribute_location);
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");

        /*
            Manage Indices for model
        */
        rust_log(&"Starting to buffer vertex & color position indices...", &"verbose_wasm_gpu_data");
        self.vertex_index_buffer = context.create_buffer();
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.vertex_index_buffer.as_ref());
        let tmp_indices: Vec<u32> = (0..(merge_vertex_and_color_data.len() / 7))
            .map(|i| i as u32)
            .collect();
        rust_log(&format!("tmp_indices is length {}", tmp_indices.len()), "super_verbose_wasm_parse");
        rust_log(&format!("merge_vertex_and_color_data is length {}", merge_vertex_and_color_data.len()), "super_verbose_wasm_parse");
        self.indices_size = tmp_indices.len();
        unsafe {
            let converted_indices = js_sys::Uint32Array::view(&tmp_indices);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &converted_indices,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        rust_log(&"...buffering complete.", &"verbose_wasm_gpu_data");
        
        return Ok(());
    }

}