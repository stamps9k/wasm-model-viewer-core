use web_sys::*;
use super::WebGl2Frame;

impl WebGl2Frame
{
    pub fn compile_shader
    (
        &self,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, String> 
    {
        let shader = self.context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.context.shader_source(&shader, source);
        self.context.compile_shader(&shader);

        if self.context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(self.context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    pub fn link_program
    (
        &mut self,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<(), String> 
    {
        let tmp = self.context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
        self.program = Some(tmp);

        self.context.attach_shader(self.program.as_mut().unwrap(), vert_shader);
        self.context.attach_shader(self.program.as_mut().unwrap(), frag_shader);
        self.context.link_program(self.program.as_mut().unwrap());

        if self.context
            .get_program_parameter(self.program.as_mut().unwrap(), WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(self.context
                .get_program_info_log(self.program.as_mut().unwrap())
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }
}
