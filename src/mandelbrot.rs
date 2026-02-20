use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

pub fn draw_mandelbrot_scene(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    elapsed_ms: f64,
) {
    let w = width.max(1.0) as u32;
    let h = height.max(1.0) as u32;

    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let scale = 3.0 / (width.min(height).max(1.0) * zoom.max(0.01));
    let center_x = -0.5 - pan_x * scale;
    let center_y = 0.0 - pan_y * scale;
    let max_iter = (12.0 + (elapsed_ms / 28.0)).min(160.0) as i32;

    for y in 0..h {
        let cy = (y as f64 - height / 2.0) * scale + center_y;
        for x in 0..w {
            let cx = (x as f64 - width / 2.0) * scale + center_x;
            let mut zx = 0.0;
            let mut zy = 0.0;
            let mut iter = 0;

            while zx * zx + zy * zy <= 4.0 && iter < max_iter {
                let xt = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = xt;
                iter += 1;
            }

            let idx = ((y * w + x) * 4) as usize;
            if iter == max_iter {
                pixels[idx] = 10;
                pixels[idx + 1] = 10;
                pixels[idx + 2] = 16;
                pixels[idx + 3] = 255;
            } else {
                let t = iter as f64 / max_iter as f64;
                let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
                let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
                let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0)
                    as u8;
                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
        }
    }

    if let Ok(image) = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&mut pixels),
        w,
        h,
    ) {
        let _ = ctx.put_image_data(&image, 0.0, 0.0);
    }
}
