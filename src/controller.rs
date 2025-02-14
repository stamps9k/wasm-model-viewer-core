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
    pub mouse_position: [f32; 2]
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
            mouse_position: [0.0, 0.0]
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

    if controller_values.rotate_x || controller_values.rotate_y || controller_values.rotate_z
    {
        m4_pretty_print("Camera Matrix", &camera_matrix);
    }

    return out;
}