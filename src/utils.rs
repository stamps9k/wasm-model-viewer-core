use crate::controller::*;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Event, EventTarget, MouseEvent };

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn get_current_time() -> f64 {
    window()
        .expect("no global `window` exists")
        .performance()
        .expect("should have `performance` available")
        .now() // Returns milliseconds since page load
}

pub fn register_get_mouse_position()
{
    //Get document
    let document = window().unwrap().document().expect("No `document` object found");

    // Convert document into an EventTarget
    let event_target: &EventTarget = document.as_ref();

    // Create a closure for the event listener
    let closure = Closure::wrap(Box::new(move |event: MouseEvent| {

        let controller_values = get_control_flags();
        let mut controller = controller_values.lock().unwrap();
        let mut update: [f32; 2] = [0.0, 0.0];
        let mouse_position = get_mouse_position(event).unwrap();
        update[0] = mouse_position.0 as f32;
        update[1] = mouse_position.1 as f32;
        controller.mouse_position = update;

    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure.forget();
}


pub fn get_mouse_position(event: MouseEvent) -> Option<(i32, i32)> {
    let mouse_event = event.dyn_ref::<MouseEvent>()?;
    Some((mouse_event.client_x(), mouse_event.client_y()))
}

//Alternative mouse position implementations. Currently expermimenting to find best type
/*
pub fn get_mouse_position_page(event: &Event) -> Option<(i32, i32)> {
    let mouse_event = event.dyn_ref::<MouseEvent>()?;
    Some((mouse_event.page_x(), mouse_event.page_y()))
}

pub fn get_mouse_position_screen(event: &Event) -> Option<(i32, i32)> {
    let mouse_event = event.dyn_ref::<MouseEvent>()?;
    Some((mouse_event.screen_x(), mouse_event.screen_y()))
}
    */

pub fn get_window_resolution() -> [f32; 2]
{
    let window = window().unwrap();

    let width = window.outer_width()
        .expect("should get window width")
        .as_f64().unwrap() as f32;

    let height = window.outer_height()
        .expect("should get window height")
        .as_f64().unwrap() as f32;

    [width, height]
}