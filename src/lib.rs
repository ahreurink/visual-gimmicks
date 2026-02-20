use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, Window};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

thread_local! {
    static STATE: RefCell<Option<Rc<RefCell<State>>>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
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
        paused: false,
    }));

    State::request_frame(&state);
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = false;
                state.start_time = 0.0;
            }
        }) as Box<dyn FnMut()>);
        let _ = restart_button.add_event_listener_with_callback(
            "click",
            cb.as_ref().unchecked_ref(),
        );
        state.borrow_mut().click_cb = Some(cb);
    }
    {
        let weak_state: Weak<RefCell<State>> = Rc::downgrade(&state);
        let cb = Closure::wrap(Box::new(move || {
            if let Some(state_rc) = weak_state.upgrade() {
                let mut state = state_rc.borrow_mut();
                state.paused = true;
            }
        }) as Box<dyn FnMut()>);
        let _ = clear_button.add_event_listener_with_callback(
            "click",
            cb.as_ref().unchecked_ref(),
        );
        state.borrow_mut().clear_cb = Some(cb);
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
    paused: bool,
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
        let (width, height) = logical_size(&self.window);
        let cx = width / 2.0;
        let cy = height / 2.0;

        let ctx = &self.ctx;
        ctx.set_fill_style(&JsValue::from_str("#0b0b0f"));
        ctx.fill_rect(0.0, 0.0, width, height);
        if self.paused {
            return;
        }

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
}

fn logical_size(window: &Window) -> (f64, f64) {
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);
    (width, height)
}

fn resize_canvas(window: &Window, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    let dpr = window.device_pixel_ratio();
    let (width, height) = logical_size(window);

    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);
    let style = canvas.style();
    let _ = style.set_property("width", &format!("{}px", width));
    let _ = style.set_property("height", &format!("{}px", height));

    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let _ = ctx.scale(dpr, dpr);
}
