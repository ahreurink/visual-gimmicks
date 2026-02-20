use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, PointerEvent, WheelEvent, Window,
};

use crate::h_fractal;
use crate::mandelbrot;
use crate::spiral;

thread_local! {
    static STATE: RefCell<Option<Rc<RefCell<State>>>> = RefCell::new(None);
}

pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id("spiral")
        .ok_or("missing canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    let restart_button = document
        .get_element_by_id("restart")
        .ok_or("missing restart button")?
        .dyn_into::<HtmlElement>()?;
    let fractal_button = document
        .get_element_by_id("fractal")
        .ok_or("missing fractal button")?
        .dyn_into::<HtmlElement>()?;
    let mandelbrot_button = document
        .get_element_by_id("mandelbrot")
        .ok_or("missing mandelbrot button")?
        .dyn_into::<HtmlElement>()?;
    let clear_button = document
        .get_element_by_id("clear")
        .ok_or("missing clear button")?
        .dyn_into::<HtmlElement>()?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into::<CanvasRenderingContext2d>()?;

    let state = Rc::new(RefCell::new(State {
        window,
        canvas,
        ctx,
        start_time: 0.0,
        raf_cb: None,
        click_cb: None,
        clear_cb: None,
        fractal_cb: None,
        mandelbrot_cb: None,
        pointer_down_cb: None,
        pointer_move_cb: None,
        pointer_up_cb: None,
        wheel_cb: None,
        paused: false,
        mode: DrawMode::Spiral,
        pan_x: 0.0,
        pan_y: 0.0,
        zoom: 1.0,
        drag_active: false,
        last_x: 0.0,
        last_y: 0.0,
    }));

    State::request_frame(&state);
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = false;
                state.start_time = 0.0;
                state.mode = DrawMode::Spiral;
            }
        }) as Box<dyn FnMut()>);
        let _ =
            restart_button.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        state.borrow_mut().click_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = false;
                state.start_time = 0.0;
                state.mode = DrawMode::HTree;
            }
        }) as Box<dyn FnMut()>);
        let _ =
            fractal_button.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        state.borrow_mut().fractal_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = false;
                state.start_time = 0.0;
                state.mode = DrawMode::Mandelbrot;
                state.pan_x = 0.0;
                state.pan_y = 0.0;
            }
        }) as Box<dyn FnMut()>);
        let _ = mandelbrot_button.add_event_listener_with_callback(
            "click",
            cb.as_ref().unchecked_ref(),
        );
        state.borrow_mut().mandelbrot_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = true;
            }
        }) as Box<dyn FnMut()>);
        let _ = clear_button.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        state.borrow_mut().clear_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let canvas = state.borrow().canvas.clone();
        let cb = Closure::wrap(Box::new(move |event: PointerEvent| {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.drag_active = true;
                state.last_x = event.client_x() as f64;
                state.last_y = event.client_y() as f64;
                let _ = canvas.set_pointer_capture(event.pointer_id());
            }
        }) as Box<dyn FnMut(PointerEvent)>);
        let _ = state
            .borrow()
            .canvas
            .add_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref());
        state.borrow_mut().pointer_down_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move |event: PointerEvent| {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                if !state.drag_active {
                    return;
                }
                let x = event.client_x() as f64;
                let y = event.client_y() as f64;
                let dx = x - state.last_x;
                let dy = y - state.last_y;
                state.pan_x += dx;
                state.pan_y += dy;
                state.last_x = x;
                state.last_y = y;
            }
        }) as Box<dyn FnMut(PointerEvent)>);
        let _ = state
            .borrow()
            .canvas
            .add_event_listener_with_callback("pointermove", cb.as_ref().unchecked_ref());
        state.borrow_mut().pointer_move_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let canvas = state.borrow().canvas.clone();
        let cb = Closure::wrap(Box::new(move |event: PointerEvent| {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.drag_active = false;
                let _ = canvas.release_pointer_capture(event.pointer_id());
            }
        }) as Box<dyn FnMut(PointerEvent)>);
        let _ = state
            .borrow()
            .canvas
            .add_event_listener_with_callback("pointerup", cb.as_ref().unchecked_ref());
        let _ = state
            .borrow()
            .canvas
            .add_event_listener_with_callback("pointercancel", cb.as_ref().unchecked_ref());
        state.borrow_mut().pointer_up_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move |event: WheelEvent| {
            if let Some(state_rc) = weak_state.upgrade() {
                event.prevent_default();
                let mut state = state_rc.borrow_mut();
                let factor = (-event.delta_y() * 0.001).exp();
                state.zoom = (state.zoom * factor).clamp(0.2, 20.0);
            }
        }) as Box<dyn FnMut(WheelEvent)>);
        let _ = state
            .borrow()
            .canvas
            .add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref());
        state.borrow_mut().wheel_cb = Some(cb);
    }

    STATE.with(|slot| {
        *slot.borrow_mut() = Some(state);
    });
    Ok(())
}

struct State {
    window: Window,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    start_time: f64,
    raf_cb: Option<Closure<dyn FnMut(f64)>>,
    click_cb: Option<Closure<dyn FnMut()>>,
    clear_cb: Option<Closure<dyn FnMut()>>,
    fractal_cb: Option<Closure<dyn FnMut()>>,
    mandelbrot_cb: Option<Closure<dyn FnMut()>>,
    pointer_down_cb: Option<Closure<dyn FnMut(PointerEvent)>>,
    pointer_move_cb: Option<Closure<dyn FnMut(PointerEvent)>>,
    pointer_up_cb: Option<Closure<dyn FnMut(PointerEvent)>>,
    wheel_cb: Option<Closure<dyn FnMut(WheelEvent)>>,
    paused: bool,
    mode: DrawMode,
    pan_x: f64,
    pan_y: f64,
    zoom: f64,
    drag_active: bool,
    last_x: f64,
    last_y: f64,
}

#[derive(Copy, Clone)]
enum DrawMode {
    Spiral,
    HTree,
    Mandelbrot,
}

impl State {
    fn request_frame(state_rc: &Rc<RefCell<Self>>) {
        if state_rc.borrow().raf_cb.is_none() {
            let weak_state: Weak<RefCell<Self>> = Rc::downgrade(state_rc);
            let cb = Closure::wrap(Box::new(move |time: f64| {
                if let Some(state_rc) = weak_state.upgrade() {
                    {
                        let mut state = state_rc.borrow_mut();
                        if state.start_time == 0.0 {
                            state.start_time = time;
                        }
                        let start_time = state.start_time;
                        state.draw(time - start_time);
                    }
                    Self::schedule_next_frame(&state_rc);
                }
            }) as Box<dyn FnMut(f64)>);
            state_rc.borrow_mut().raf_cb = Some(cb);
        }

        Self::schedule_next_frame(state_rc);
    }

    fn schedule_next_frame(state_rc: &Rc<RefCell<Self>>) {
        let state = state_rc.borrow();
        if let Some(cb) = state.raf_cb.as_ref() {
            let _ = state
                .window
                .request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }

    fn draw(&mut self, elapsed_ms: f64) {
        resize_canvas(&self.window, &self.canvas, &self.ctx);
        let (width, height) = logical_size(&self.window, &self.canvas);
        let cx = width / 2.0;
        let cy = height / 2.0;

        let ctx = &self.ctx;
        ctx.set_fill_style(&JsValue::from_str("#0b0b0f"));
        ctx.fill_rect(0.0, 0.0, width, height);
        if self.paused {
            return;
        }

        match self.mode {
            DrawMode::Spiral => {
                ctx.save();
                let _ = ctx.translate(self.pan_x, self.pan_y);
                let _ = ctx.translate(cx, cy);
                let _ = ctx.scale(self.zoom, self.zoom);
                let _ = ctx.translate(-cx, -cy);
                spiral::draw_spiral(ctx, cx, cy, width, height, elapsed_ms);
                let _ = ctx.restore();
            }
            DrawMode::HTree => {
                ctx.save();
                let _ = ctx.translate(self.pan_x, self.pan_y);
                let _ = ctx.translate(cx, cy);
                let _ = ctx.scale(self.zoom, self.zoom);
                let _ = ctx.translate(-cx, -cy);
                h_fractal::draw_h_tree_scene(ctx, cx, cy, width, height, elapsed_ms);
                let _ = ctx.restore();
            }
            DrawMode::Mandelbrot => {
                mandelbrot::draw_mandelbrot_scene(
                    ctx,
                    width,
                    height,
                    self.pan_x,
                    self.pan_y,
                    self.zoom,
                    elapsed_ms,
                );
            }
        }
    }
}

fn logical_size(window: &Window, canvas: &HtmlCanvasElement) -> (f64, f64) {
    let width = canvas.client_width() as f64;
    let height = canvas.client_height() as f64;
    if width > 0.0 && height > 0.0 {
        return (width, height);
    }

    let fallback_width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let fallback_height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);
    (fallback_width, fallback_height)
}

fn resize_canvas(window: &Window, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    let dpr = window.device_pixel_ratio();
    let (width, height) = logical_size(window, canvas);

    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);
    let style = canvas.style();
    let _ = style.set_property("width", &format!("{}px", width));
    let _ = style.set_property("height", &format!("{}px", height));

    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let _ = ctx.scale(dpr, dpr);
}
