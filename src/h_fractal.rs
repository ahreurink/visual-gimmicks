use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub fn draw_h_tree_scene(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    width: f64,
    height: f64,
    elapsed_ms: f64,
) {
    let base = 0.55 * width.min(height);
    let depth = ((elapsed_ms / 500.0).floor() as i32).clamp(0, 6);
    ctx.set_stroke_style(&JsValue::from_str("#e9e1ff"));
    ctx.set_line_width(2.0);
    draw_h_tree(ctx, cx, cy, base, depth);
}

fn draw_h_tree(ctx: &CanvasRenderingContext2d, x: f64, y: f64, len: f64, depth: i32) {
    if depth < 0 || len < 2.0 {
        return;
    }

    let half = len / 2.0;
    let x0 = x - half;
    let x1 = x + half;
    let y0 = y - half;
    let y1 = y + half;

    ctx.begin_path();
    ctx.move_to(x0, y0);
    ctx.line_to(x0, y1);
    ctx.move_to(x1, y0);
    ctx.line_to(x1, y1);
    ctx.move_to(x0, y);
    ctx.line_to(x1, y);
    ctx.stroke();

    if depth == 0 {
        return;
    }

    let next = len / 2.0;
    draw_h_tree(ctx, x0, y0, next, depth - 1);
    draw_h_tree(ctx, x0, y1, next, depth - 1);
    draw_h_tree(ctx, x1, y0, next, depth - 1);
    draw_h_tree(ctx, x1, y1, next, depth - 1);
}
