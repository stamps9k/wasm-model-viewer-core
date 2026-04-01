use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub fn rust_log(message: &str, level:&str)
{
	let logger_api = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str(&format!("wasm_logging_api")))
        .expect("function wasm_logging_api not found in global scope.");

	if let Some(logger_api_fn) = logger_api.dyn_ref::<js_sys::Function>() {
        let _ = logger_api_fn.call2(&JsValue::NULL, &JsValue::from_str(message), &JsValue::from_str(level));
    } else {
        panic!("wasm_logging_api is not a function");
    }	
}

pub fn m4_pretty_print_verbose(name: &str, matrix: &[f32; 16])
{
	rust_log(&format!("Matrix is {}:", name), &"verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[0], matrix[4], matrix[8], matrix[12]), &"verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[1], matrix[5], matrix[9], matrix[13]), &"verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[2], matrix[6], matrix[10], matrix[14]), &"verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[3], matrix[7], matrix[11], matrix[15]), &"verbose_wasm_math");
}

pub fn m4_pretty_print_super_verbose(name: &str, matrix: &[f32; 16])
{
	rust_log(&format!("Matrix is {}:", name), &"super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[0], matrix[4], matrix[8], matrix[12]), &"super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[1], matrix[5], matrix[9], matrix[13]), &"super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[2], matrix[6], matrix[10], matrix[14]), &"super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[3], matrix[7], matrix[11], matrix[15]), &"super_verbose_wasm_math");
}

pub fn m4_pretty_print_super_super_verbose(name: &str, matrix: &[f32; 16])
{
	rust_log(&format!("Matrix is {}:", name), &"verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[0], matrix[4], matrix[8], matrix[12]), &"super_super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[1], matrix[5], matrix[9], matrix[13]), &"super_super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[2], matrix[6], matrix[10], matrix[14]), &"super_super_verbose_wasm_math");
	rust_log(&format!("{}, {}, {}, {}", matrix[3], matrix[7], matrix[11], matrix[15]), &"super_super_verbose_wasm_math");
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