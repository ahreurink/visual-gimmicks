use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub fn draw_spiral(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    width: f64,
    height: f64,
    elapsed_ms: f64,
) {
    let max_t = 80.0 * std::f64::consts::PI;
    let duration = 4200.0;
    let progress = (elapsed_ms / duration).min(1.0);
    let t_end = max_t * progress;

    let a = 2.2;
    let b = 4.2;
    let max_r = (a + b * max_t).max(1.0);
    let scale = 0.45 * width.min(height) / max_r;

    let mut t = 0.0;
    let dt = 0.08;
    while t < t_end {
        let t2 = (t + dt).min(t_end);
        let r1 = a + b * t;
        let r2 = a + b * t2;
        let x1 = cx + scale * r1 * t.cos();
        let y1 = cy + scale * r1 * t.sin();
        let x2 = cx + scale * r2 * t2.cos();
        let y2 = cy + scale * r2 * t2.sin();

        let hue = (t / max_t) * 360.0;
        let color = format!("hsl({:.0}, 95%, 60%)", hue);
        ctx.set_stroke_style(&JsValue::from_str(&color));
        ctx.set_line_width(2.2);
        ctx.begin_path();
        ctx.move_to(x1, y1);
        ctx.line_to(x2, y2);
        ctx.stroke();

        t = t2;
    }
}
