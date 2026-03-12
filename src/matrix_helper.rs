use webgl_matrix::*;

pub fn scaling_matrix(factor: f32) -> Mat4
{
    let scale = [
        factor, 0.0, 0.0, 0.0,
        0.0, factor, 0.0, 0.0,
        0.0, 0.0, factor, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];
    return scale
}