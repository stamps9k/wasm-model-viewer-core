use js_sys::*;
use std::io::Cursor;
use image::ImageReader;

use crate::logger::*;

use super::WebGl2WavefrontObject;

impl WebGl2WavefrontObject
{   
    /*
	*
	*	Convert the passed base64 image data into a raw u8 array of rbga values. 
	*
	*/
	pub(in super) fn create_image_as_uint8_array(&mut self) -> Result<Uint8Array, String> 
	{
		//For now assuming that there is only ever 1 texture per object
		let texture_b64 = self.textures.clone().unwrap().into_values().next().expect("Expected at least one texture");

		// Convert base64 to a binary array
		let bytes = base64::decode(texture_b64.clone()).map_err(|_| "Failed to decode base64")?;

		let img1 = match ImageReader::new(Cursor::new(bytes)).with_guessed_format()
		{
			Ok(img1) => img1,
			Err(e) => return Err(e.to_string())
		};

		let img2 = match img1.decode()
		{
			Ok(img2) => img2,
			Err(e) => return Err(e.to_string())
		};

		let rgba_img = img2.to_rgba8();

		// Get image dimensions
		let (width, height) = rgba_img.dimensions();
		self.texture_height = height as i32;
		self.texture_width = width as i32;
		rust_log(&format!("Image size: {} x {}", width, height), &"info_wasm_parse");

		// Access raw pixel data
		let pixels = rgba_img.as_raw();

		// Create a Blob from binary data
		let array = js_sys::Uint8Array::from(pixels.as_slice());

		self.log_js_uint8_array(&array);

		return Ok(array);	
	}

    /*
	*
	*	Log the RGBA elements in image array
	*
	*/
	pub(in super) fn log_js_uint8_array(&mut self, array: &js_sys::Uint8Array)
	{
		rust_log(&"Loaded texure coordinates are:", &"super_super_verbose_wasm_parse");
		rust_log(&(array.to_string().as_string().unwrap()), &"super_super_verbose_wasm_parse");
	}
}
