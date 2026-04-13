use crate::controller::*;
use crate::logger::*;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{window, EventTarget, MouseEvent, WheelEvent, KeyboardEvent, HtmlCanvasElement };
use js_sys::Array;
use js_sys::Map;
use std::collections::HashMap;

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

        //Update previous mouse position with current position
        controller.previous_mouse_position = controller.current_mouse_position;

        //Get the current values and set in rust
        let mut update: [f32; 2] = [0.0, 0.0];
        let mouse_position = get_mouse_position(event).unwrap();
        update[0] = mouse_position.0 as f32;
        update[1] = mouse_position.1 as f32;
        controller.current_mouse_position = update;
        controller.mouse_moving = true;

        //Log new values
        rust_log(&format!("Previous mouse position is: {}, {}", controller.previous_mouse_position[0], controller.previous_mouse_position[1]), "super_super_verbose_wasm_scene");
        rust_log(&format!("Current mouse position is: {}, {}", controller.current_mouse_position[0], controller.current_mouse_position[1]), "super_super_verbose_wasm_scene");
        rust_log
        (
            &format!
            (
                "Mouse delta is: {}, {}", 
                controller.current_mouse_position[0] - controller.previous_mouse_position[0], 
                controller.current_mouse_position[1] - controller.previous_mouse_position[1]
            ), 
            "super_super_verbose_wasm_scene"
        );
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

pub fn register_mouse_down()
{
    //Get document
    let document = window().unwrap().document().expect("No `document` object found");

    // Convert document into an EventTarget
    let event_target: &EventTarget = document.as_ref();

    // Create a closure for the event listener when mouse is pressed down
    let closure_mouse_down = Closure::wrap(Box::new(move |event: MouseEvent| {
        let controller_values = get_control_flags();
                let mut controller = controller_values.lock().unwrap();

        let mouse_event = event.dyn_ref::<MouseEvent>().unwrap();
        if mouse_event.button() == 0
        {
            //Log new values
            rust_log("Mouse button 0 down.", "super_verbose_wasm_scene");
            controller.mouse_0_down = true;
        } 
    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("mousedown", closure_mouse_down.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure_mouse_down.forget();
}

pub fn register_mouse_up()
{
    //Get document
    let document = window().unwrap().document().expect("No `document` object found");

    // Convert document into an EventTarget
    let event_target: &EventTarget = document.as_ref();

    // Create a closure for the event listener when mouse is pressed down
    let closure_mouse_up = Closure::wrap(Box::new(move |event: MouseEvent| {
        let controller_values = get_control_flags();
                let mut controller = controller_values.lock().unwrap();

        let mouse_event = event.dyn_ref::<MouseEvent>().unwrap();
        if mouse_event.button() == 0
        {
            controller.mouse_0_down = false;
            rust_log("Mouse button 0 up.", "super_verbose_wasm_scene");
        } 
    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("mouseup", closure_mouse_up.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure_mouse_up.forget();
}

pub fn register_mouse_wheel()
{
    //Get document
    let canvas = window().unwrap().document().unwrap().get_element_by_id("glCanvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    // Convert document into an EventTarget
    let event_target: &EventTarget = canvas.as_ref();

    // Create a closure for the event listener when mouse is pressed down
    let closure_mouse_wheel = Closure::wrap(Box::new(move |event: WheelEvent| {
        let controller_values = get_control_flags();
                let mut controller = controller_values.lock().unwrap();

        let wheel_event = event.dyn_ref::<WheelEvent>().unwrap();

        controller.wheel_delta = [wheel_event.delta_x() as f32, wheel_event.delta_y() as f32];
        controller.wheel_scroll = true;


        if wheel_event.shift_key()
        {
            // Regular scroll with shift button active.
            wheel_event.prevent_default();
            rust_log(&format!("Shift scroll size {}, {} registered", wheel_event.delta_x(), wheel_event.delta_y()), "super_verbose_wasm_scene");
        } 
        else if wheel_event.ctrl_key()
        {
            // Pinch-to-zoom gesture (browser sets ctrlKey=true for this). Can also be regular scroll with ctrl key pressed
            wheel_event.prevent_default();
            rust_log(&format!("Control scroll size {}, {} registered. Maybe pinch gesture JS cannot differentiate.", wheel_event.delta_x(), wheel_event.delta_y()), "super_verbose_wasm_scene");
        }
        else
        {
            // Regular scroll / two-finger pan
            rust_log(&format!("Unmodified Wheel scroll size {}, {} registered", wheel_event.delta_x(), wheel_event.delta_y()), "super_verbose_wasm_scene");
        }

    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("wheel", closure_mouse_wheel.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure_mouse_wheel.forget();
}

pub fn register_key_down()
{
    //Get document
    let document = window().unwrap().document().expect("No `document` object found");

    // Convert document into an EventTarget
    let event_target: &EventTarget = document.as_ref();

    // Create a closure for the event listener when mouse is pressed down
    let closure_key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let controller_values = get_control_flags();
                let mut controller = controller_values.lock().unwrap();

        let key_event = event.dyn_ref::<KeyboardEvent>().unwrap();

        if event.repeat() { return; } //Ignore repeat key presses.

        match key_event.code().as_str() {
            "ShiftLeft" => { controller.shift_key = true }
            "ControlLeft" => { controller.ctrl_key = true }
            _ => { /* Default does nothing */ }
        }

        rust_log(&format!("Keyboard {} pressed.", key_event.code()), "super_verbose_wasm_scene");

    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("keydown", closure_key_down.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure_key_down.forget();
}

pub fn register_key_up()
{
    //Get document
    let document = window().unwrap().document().expect("No `document` object found");

    // Convert document into an EventTarget
    let event_target: &EventTarget = document.as_ref();

    // Create a closure for the event listener when mouse is pressed down
    let closure_key_down = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let controller_values = get_control_flags();
                let mut controller = controller_values.lock().unwrap();

        let key_event = event.dyn_ref::<KeyboardEvent>().unwrap();

        match key_event.code().as_str() {
            "ShiftLeft" => { controller.shift_key = false }
            "CtrlLeft" => { controller.ctrl_key = false }
            _ => { /* Default does nothing */ }
        }

        rust_log(&format!("Keyboard {} released.", key_event.code()), "super_verbose_wasm_scene");

    }) as Box<dyn FnMut(_)>);

    // Attach event listener
    event_target
        .add_event_listener_with_callback("keyup", closure_key_down.as_ref().unchecked_ref())
        .expect("Failed to add event listener");

    // Prevent Rust from dropping the closure
    closure_key_down.forget();
}

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

pub fn get_js_sys_map_to_hashmap(outer_map: &Map, inner_map_key: &str) -> Option<HashMap<String, String>>
{
    let inner_map_jsvalue = outer_map.get(&JsValue::from_str(inner_map_key));
    let inner_map = Map::from(inner_map_jsvalue);
    let result: HashMap<String, String> = Array::from(&inner_map.entries())
        .iter()
        .filter_map(|entry| {
            let pair = Array::from(&entry);
            let key = pair.get(0).as_string()?;
            let value = pair.get(1).as_string()?;
            Some((key, value))
        })
        .collect();

    if result.is_empty()
    {
        return None
    }
    else
    {
        return Some(result);
    }
}