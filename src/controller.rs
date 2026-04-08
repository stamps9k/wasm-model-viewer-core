use crate::logger::*;

use std::f64::consts::*;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::Mutex;
use webgl_matrix::*;

static CONTROL_FLAGS: OnceLock<Arc<Mutex<ControllerValues>>> = OnceLock::new();

#[derive(Clone)]
pub struct ControllerValues
{
    pub rotate_x: bool,
    pub rotate_y: bool,
    pub rotate_z: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub mouse_moving: bool,
    pub current_mouse_position: [f32; 2],
    pub previous_mouse_position: [f32; 2],
    pub mouse_0_down: bool
}

impl ControllerValues
{
    pub fn new() -> Self 
    {
        Self 
        {
            rotate_x: false,
            rotate_y: false,
            rotate_z: false,
            zoom_in: false,
            zoom_out: false,
            mouse_moving: false,
            current_mouse_position: [0.0, 0.0],
            previous_mouse_position: [0.0, 0.0],
            mouse_0_down: false
        }
    }
}

pub fn get_control_flags() -> Arc<Mutex<ControllerValues>> 
{
    CONTROL_FLAGS
        .get_or_init(|| Arc::new(Mutex::new(ControllerValues::new())))
        .clone()
}

pub fn update_model_matrix(model_matrix: &Mat4, controller_values: &ControllerValues) -> Mat4
{
    let mut out = model_matrix.clone();

    let rotation_angle: f32 = (PI / 180.0) as f32;

    if controller_values.mouse_0_down && controller_values.mouse_moving
    {
        rust_log
        (
            &format!
            (
                "Mouse delta is: {}, {}", 
                controller_values.current_mouse_position[0] - controller_values.previous_mouse_position[0], 
                controller_values.current_mouse_position[1] - controller_values.previous_mouse_position[1]
            ), 
            "super_super_verbose_wasm_scene"
        );
        
        out = calculate_mouse_rotation(model_matrix, &controller_values).unwrap_or(out);
    }

    if controller_values.rotate_x
    {
        let rotation_axis: [f32; 3] = [1.0, 0.0, 0.0]; 
        out.rotate(rotation_angle, &rotation_axis);
    } 
    if controller_values.rotate_y
    {
        let rotation_axis: [f32; 3] = [0.0, 1.0, 0.0]; 
        out.rotate(rotation_angle, &rotation_axis);
    } 
    if controller_values.rotate_z
    {
        let rotation_axis: [f32; 3] = [0.0, 0.0, 1.0]; 
        out.rotate(rotation_angle, &rotation_axis);
    }

    if controller_values.rotate_x || 
        controller_values.rotate_y || 
        controller_values.rotate_z || 
        controller_values.zoom_in || 
        controller_values.zoom_out
    {
        m4_pretty_print_super_super_verbose("Model Matrix", &out);
    }

    return out;
}

pub fn update_camera_position(camera_matrix: &Mat4, controller_values: &ControllerValues) -> Mat4
{
    let mut out = camera_matrix.clone();

    if controller_values.zoom_in
    {
        // Extract forward vector from camera matrix (third column)
        let forward: Vec3 = [camera_matrix[2], camera_matrix[6], camera_matrix[10]];

        // Normalise it manually using .mag()
        let magnitude = forward.mag();
        let normalised = forward.scale(1.0 / magnitude);

        // Scale by movement speed
        let movement = normalised.scale(0.1);

        // Apply to camera
        out.translate(&movement);
    }

    if controller_values.zoom_out
    {
        // Extract forward vector from camera matrix (third column)
        let forward: Vec3 = [camera_matrix[2], camera_matrix[6], camera_matrix[10]];

        // Normalise it manually using .mag()
        let magnitude = forward.mag();
        let normalised = forward.scale(1.0 / magnitude);

        // Scale by movement speed
        let movement = normalised.scale(-0.1);

        // Apply to camera
        out.translate(&movement);
    }

    if controller_values.rotate_x || 
        controller_values.rotate_y || 
        controller_values.rotate_z || 
        controller_values.zoom_in || 
        controller_values.zoom_out
    {
        m4_pretty_print_super_super_verbose("Camera Matrix", &out);
    }

    return out;
}

fn calculate_mouse_rotation(model_matrix: &Mat4, controller_values: &ControllerValues) -> Result<Mat4, String>
{
    let mut delta = Mat4::identity();
    let angle: f32 = ((PI / 180.0) * 5.0) as f32;

    let dx = controller_values.current_mouse_position[0] - controller_values.previous_mouse_position[0];
    let dy = controller_values.current_mouse_position[1] - controller_values.previous_mouse_position[1];

    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return Err(String::from("Mouse movement too small to be calculated")); // skip this frame, no meaningful movement
    }

    //Get angle of rotation
    let rotation_axis: Vec3 = [dy, dx, 0.0];
    let magnitude = rotation_axis.mag();
    let normalised: [f32; 3] = rotation_axis.scale(1.0 / magnitude);

    //Rotate
    delta.rotate(angle, &normalised);

    //Copy the model_matrix and apply the rotation
    let mut out = *model_matrix;
    out.mul(&delta);
    
    //Return
    return Ok(out);
}