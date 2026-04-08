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

pub fn update_camera_position(camera_matrix: &Mat4, controller_values: &ControllerValues) -> Mat4
{
    let mut out = camera_matrix.clone();

    let rotation_angle: f32 = (PI / 180.0) as f32;

    if controller_values.mouse_0_down
    {
        rust_log(&"Mouse is down", &"info_wasm_math");
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
        m4_pretty_print_super_super_verbose("Camera Matrix", &camera_matrix);
    }

    return out;
}