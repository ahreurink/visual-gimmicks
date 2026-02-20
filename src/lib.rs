mod app;
mod h_fractal;
mod mandelbrot;
mod spiral;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    app::start()
}
