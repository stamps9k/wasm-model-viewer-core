use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Rust function that dynamically calls super_verbose_api from the JS global scope
#[wasm_bindgen]
pub fn rust_super_super_verbose(message: &str) 
{
    let super_super_verbose_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("super_super_verbose_api"))
        .expect("super_super_verbose_api function not found in global scope");

    if let Some(super_super_verbose_api_fn) = super_super_verbose_api.dyn_ref::<js_sys::Function>() {
        let _ = super_super_verbose_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("super_super_verbose_api is not a function");
    }
}


// Rust function that dynamically calls super_verbose_api from the JS global scope
#[wasm_bindgen]
pub fn rust_super_verbose(message: &str) 
{
    let super_verbose_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("super_verbose_api"))
        .expect("super_verbose_api function not found in global scope");

    if let Some(super_verbose_api_fn) = super_verbose_api.dyn_ref::<js_sys::Function>() {
        let _ = super_verbose_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("super_verbose_api is not a function");
    }
}

// Rust function that dynamically calls verbose_api from the JS global scope
#[wasm_bindgen]
pub fn rust_verbose(message: &str) 
{
    let verbose_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("verbose_api"))
        .expect("verbose_api function not found in global scope");

    if let Some(verbose_api_fn) = verbose_api.dyn_ref::<js_sys::Function>() {
        let _ = verbose_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("verbose_api is not a function");
    }
}

// Rust function that dynamically calls info_api from the JS global scope
#[wasm_bindgen]
pub fn rust_info(message: &str) 
{
    let info_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("info_api"))
        .expect("info_api function not found in global scope");

    if let Some(info_api_fn) = info_api.dyn_ref::<js_sys::Function>() {
        let _ = info_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("info_api is not a function");
    }
}

// Rust function that dynamically calls warn_api from the JS global scope
#[wasm_bindgen]
pub fn rust_warn(message: &str) 
{
    let warn_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("warn_api"))
        .expect("warn_api function not found in global scope");

    if let Some(warn_api_fn) = warn_api.dyn_ref::<js_sys::Function>() {
        let _ = warn_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("warn_api is not a function");
    }
}

// Rust function that dynamically calls error_api from the JS global scope
#[wasm_bindgen]
pub fn rust_error(message: &str) 
{
    let error_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("error_api"))
        .expect("error_api function not found in global scope");

    if let Some(error_api_fn) = error_api.dyn_ref::<js_sys::Function>() {
        let _ = error_api_fn.call1(&JsValue::NULL, &JsValue::from_str(message));
    } else {
        panic!("error_api is not a function");
    }
}

pub fn m4_pretty_print(name: &str, matrix: &[f32; 16])
{
	rust_super_super_verbose(&("Matrix is ".to_owned() + name + ":"));
	rust_super_super_verbose
	(
		&(
			matrix[0].to_string().as_str().to_owned() + ", " +
			matrix[4].to_string().as_str() + ", " +
			matrix[8].to_string().as_str() + ", " +
			matrix[12].to_string().as_str() 
		)
	);
	rust_super_super_verbose
	(
		&(
			matrix[1].to_string().as_str().to_owned() + ", " +
			matrix[5].to_string().as_str() + ", " +
			matrix[9].to_string().as_str() + ", " +
			matrix[13].to_string().as_str() 
		)
	);
	rust_super_super_verbose
	(
		&(
			matrix[2].to_string().as_str().to_owned() + ", " +
			matrix[6].to_string().as_str() + ", " +
			matrix[10].to_string().as_str() + ", " +
			matrix[14].to_string().as_str() 
		)
	);
	rust_super_super_verbose
	(
		&(
			matrix[3].to_string().as_str().to_owned() + ", " +
			matrix[7].to_string().as_str() + ", " +
			matrix[11].to_string().as_str() + ", " +
			matrix[15].to_string().as_str() 
		)
	);
}

#[wasm_bindgen]
pub fn set_fps(fps: f64)
{
    let set_fps_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("set_fps"))
        .expect("set_fps function not found in global scope");
    if let Some(fps_api_fn) = set_fps_api.dyn_ref::<js_sys::Function>() {
        let _ = fps_api_fn.call1(&JsValue::NULL, &JsValue::from(fps));
    } else {
        panic!("set_fps is not a function");
    }
}