use std::rc::Rc;
use std::cell::RefCell;
//use webgl_matrix::*;
use wasm_bindgen::prelude::*;
use math::mean;

use crate::update_camera_position;
use crate::utils::*;
use crate::logger::*;
use crate::controller::*;

use super::WebGl2Frame;

pub fn initialize_animation(frame_wrap: &Rc<RefCell<WebGl2Frame>>) 
{
	//Clone the frame for the closure
	let frame_closure = Rc::clone(frame_wrap);

	//Clone the frame for the intial animation frame request
	let frame_animation = Rc::clone(frame_wrap);

	//Closure variables
	let f = Rc::new(RefCell::new(None));
	let g = f.clone();

	//FPS calculator variables
	let mut base: f64 = get_current_time();
	let mut frames_delta: [f64; 10] = [0.0; 10];

	//Initial time tracker
	let mut time: f32 = 1.0;

	let mut i: f32 = 0.0;
	*g.borrow_mut() = Some(Closure::new(move || {	
		//Movement variables
		let tmp2 = get_control_flags();
		let controller_values = tmp2.lock().unwrap();
		
		//FPS Caclulator
		let now = get_current_time();
		match i as i32 % 10 
		{
			0 => frames_delta[0] = now - base,
			1 => frames_delta[1] = now - base,
			2 => frames_delta[2] = now - base,
			3 => frames_delta[3] = now - base,
			4 => frames_delta[4] = now - base,
			5 => frames_delta[5] = now - base,
			6 => frames_delta[6] = now - base,
			7 => frames_delta[7] = now - base,
			8 => frames_delta[8] = now - base,
			9 => 
			{
				frames_delta[9] = now - base;
				base = get_current_time();
				let fps: f64 = mean::arithmetic(&frames_delta);
				set_fps(fps);
			},
			_ => panic!("Don't know how you got here!")
		}

		{
			//Borrow the Rc as a mutable for use in the animation
			let mut frame = frame_closure.borrow_mut();

			//Update the camera position
			frame.camera_matrix = update_camera_position(&frame.camera_matrix, &controller_values);

			//Mutable reference to the Webgl Frame
			let tmp = frame.program.as_mut().unwrap().clone();

			//Pass worldspace transfomration to the GPU
			let position_index = frame.context.get_uniform_location(&tmp, "u_camera_matrix");
			frame.context.uniform_matrix4fv_with_f32_array(position_index.as_ref(), false, &frame.camera_matrix);
			
			//Pass mouse position to the GPU
			let mouse_position = controller_values.mouse_position;
			rust_super_super_verbose
			(
				&(
					"Passing u_mouse_position ".to_owned() + 
					mouse_position[0].to_string().as_str() + " x " + mouse_position[1].to_string().as_str() + 
					" to shader"
				)
			);
			let mouse_position_index = frame.context.get_uniform_location(&tmp, "u_mouse_position");
			frame.context.uniform2fv_with_f32_array(mouse_position_index.as_ref(), &mouse_position);

			//Pass time to the GPU
			time = time + ((now - base) / 1000.0) as f32;
			rust_super_super_verbose(&("Passing u_time ".to_owned() + time.to_string().as_str() + " to shader."));
			let time_index = frame.context.get_uniform_location(&tmp, "u_time");
			frame.context.uniform1f(time_index.as_ref(), time);

			frame.draw();

			// Set the body's text content to how many times this
			// requestAnimationFrame callback has fired.
			i += 1.0;

			// Schedule ourself for another requestAnimationFrame callback.
			frame.request_animation_frame(f.borrow().as_ref().unwrap());
		}
	}));

	frame_animation.borrow_mut().request_animation_frame(g.borrow().as_ref().unwrap());
}