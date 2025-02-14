use web_sys::*;
use crate::webgl::WebGl2Frame;

pub fn compile_shader(
    frame: &WebGl2Frame,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = frame.context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    frame.context.shader_source(&shader, source);
    frame.context.compile_shader(&shader);

    if frame.context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(frame.context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    frame: &mut WebGl2Frame,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<(), String> {
    let tmp = frame.context
    .create_program()
    .ok_or_else(|| String::from("Unable to create shader object"))?;
    frame.program = Some(tmp);

    frame.context.attach_shader(&frame.program.as_mut().unwrap(), vert_shader);
    frame.context.attach_shader(&frame.program.as_mut().unwrap(), frag_shader);
    frame.context.link_program(&frame.program.as_mut().unwrap());

    if frame.context
        .get_program_parameter(&frame.program.as_mut().unwrap(), WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(frame.context
            .get_program_info_log(&frame.program.as_mut().unwrap())
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
