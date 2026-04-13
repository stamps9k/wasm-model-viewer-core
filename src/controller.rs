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
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub wheel_scroll: bool,
    pub wheel_delta: [f32; 2],
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
            shift_key: false,
            ctrl_key: false,
            wheel_scroll: false,
            wheel_delta: [0.0, 0.0],
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

pub fn update_camera_position(camera_matrix: &Mat4, controller_values: &mut ControllerValues) -> Mat4
{
    let mut out = camera_matrix.clone();

    // Update the camera for zoom_in check box
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

    // Update the camera for zoom out check box
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

    // Update the camera for mouse wheel zoom
    if controller_values.shift_key && controller_values.wheel_scroll
    {
        // Extract forward vector from camera matrix (third column)
        let forward: Vec3 = [camera_matrix[2], camera_matrix[6], camera_matrix[10]];

        // Normalise it manually using .mag()
        let magnitude = forward.mag();
        let normalised = forward.scale(1.0 / magnitude);

        // Scale by wheel delta
        let movement = normalised.scale(controller_values.wheel_delta[1] * 0.01);

        // Apply to camera
        out.translate(&movement);

        // Drop wheel data after handling
        controller_values.wheel_scroll = false;
        controller_values.wheel_delta = [0.0, 0.0];
    }

    //Update the camera for whose wheel pan
    if controller_values.ctrl_key && controller_values.wheel_scroll
    {
        // Extract horizontal vector from camera matrix (first column) and normalise
        let horizontal: Vec3 = [camera_matrix[0], camera_matrix[4], camera_matrix[8]];
        let magnitude_h = horizontal.mag();
        let normalised_h = horizontal.scale(1.0 / magnitude_h);

        // Extract vertical vector from camera matrix (second column) and normalise
        let vertical: Vec3 = [camera_matrix[1], camera_matrix[5], camera_matrix[9]];
        let magnitude_v = vertical.mag();
        let normalised_v = vertical.scale(1.0 / magnitude_v);


        // Scale by wheel delta
        let movement_h = normalised_h.scale(controller_values.wheel_delta[0] * -0.01);
        let movement_v = normalised_v.scale(controller_values.wheel_delta[1] * 0.01);

        // Apply to camera
        out.translate(&movement_h);
        out.translate(&movement_v);

        // Drop wheel data after handling
        controller_values.wheel_scroll = false;
        controller_values.wheel_delta = [0.0, 0.0];
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