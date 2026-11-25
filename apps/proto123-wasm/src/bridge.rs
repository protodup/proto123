//! wasm-bindgen exports and the Ratzilla canvas. Ratzilla is used for drawing only: all input
//! arrives as raw primitives from `web/host.js` and is interpreted by the shared core.

use crate::translate;
use proto123_engine::{Redraw, Session};
use proto123_theme::high_contrast;
use proto123_tui::{InputMapper, LayoutMap, render};
use ratatui::Terminal;
use ratzilla::backend::canvas::{CanvasBackend, CanvasBackendOptions};
use ratzilla::web_sys;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

struct App {
    parent_id: String,
    session: Session,
    mapper: InputMapper,
    layout: LayoutMap,
    terminal: Terminal<CanvasBackend>,
    css_size: (u32, u32),
    dpr: f64,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn js_err(message: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&message.to_string())
}

fn combine(a: Redraw, b: Redraw) -> Redraw {
    match (a, b) {
        (Redraw::Full, _) | (_, Redraw::Full) => Redraw::Full,
        (Redraw::Regions, _) | (_, Redraw::Regions) => Redraw::Regions,
        _ => Redraw::None,
    }
}

/// Ratzilla appends a new canvas to its parent on every construction, so the previous one is
/// removed first to keep exactly one `<canvas>` in the page (FR-037, SC-009).
fn remove_previous_canvas(parent_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| js_err("no document"))?;
    if let Some(parent) = document.get_element_by_id(parent_id)
        && let Some(child) = parent.last_element_child()
    {
        parent.remove_child(&child)?;
    }
    Ok(())
}

fn make_terminal(parent_id: &str, css: (u32, u32), dpr: f64) -> Result<Terminal<CanvasBackend>, JsValue> {
    remove_previous_canvas(parent_id)?;
    let px_w = (f64::from(css.0) * dpr).round() as u32;
    let px_h = (f64::from(css.1) * dpr).round() as u32;
    let options = CanvasBackendOptions::new().grid_id(parent_id).size((px_w, px_h));
    let backend = CanvasBackend::new_with_options(options).map_err(js_err)?;
    Terminal::new(backend).map_err(js_err)
}

/// Draws one frame and records its layout map for hit-testing.
fn draw(app: &mut App, full: bool) -> Result<(), JsValue> {
    if full {
        app.terminal.clear().map_err(js_err)?;
    }
    let mut next = LayoutMap::default();
    let session = &app.session;
    app.terminal
        .draw(|frame| {
            next = render(session, frame.area(), frame.buffer_mut());
        })
        .map_err(js_err)?;
    app.layout = next;
    Ok(())
}

fn apply_raw(app: &mut App, event: proto123_tui::RawEvent) -> Result<(), JsValue> {
    let mut redraw = Redraw::None;
    for command in app.mapper.map(event, &app.layout, &app.session) {
        redraw = combine(redraw, app.session.dispatch(command).redraw);
    }
    match redraw {
        Redraw::None => Ok(()),
        Redraw::Regions => draw(app, false),
        Redraw::Full => draw(app, true),
    }
}

/// Starts the app inside `parent_id` at the given CSS size and device pixel ratio.
#[wasm_bindgen]
pub fn start(parent_id: &str, css_w: u32, css_h: u32, dpr: f64) -> Result<(), JsValue> {
    let terminal = make_terminal(parent_id, (css_w, css_h), dpr)?;
    let mut app = App {
        parent_id: parent_id.to_owned(),
        session: Session::new(high_contrast()),
        mapper: InputMapper::new(proto123_theme::GridMetrics::DEFAULT),
        layout: LayoutMap::default(),
        terminal,
        css_size: (css_w, css_h),
        dpr,
    };
    let resize = translate::resize(f64::from(css_w), f64::from(css_h), dpr, 0);
    apply_raw(&mut app, resize)?;
    APP.with(|cell| *cell.borrow_mut() = Some(app));
    Ok(())
}

/// Forwards one `keydown`. Returns nothing; the core decides what changes.
#[wasm_bindgen]
pub fn on_key(key: &str, shift: bool, ctrl: bool, alt: bool, meta: bool, t_ms: f64) -> Result<(), JsValue> {
    let Some(raw) = translate::keydown(key, shift, ctrl, alt, meta, t_ms as u64) else {
        return Ok(());
    };
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => apply_raw(app, raw),
        None => Ok(()),
    })
}

/// Forwards a canvas size or device-pixel-ratio change; rebuilds the canvas when it changed.
#[wasm_bindgen]
pub fn on_resize(css_w: u32, css_h: u32, dpr: f64, t_ms: f64) -> Result<(), JsValue> {
    APP.with(|cell| {
        let mut slot = cell.borrow_mut();
        let Some(app) = slot.as_mut() else {
            return Ok(());
        };
        if app.css_size != (css_w, css_h) || (app.dpr - dpr).abs() > f64::EPSILON {
            app.terminal = make_terminal(&app.parent_id, (css_w, css_h), dpr)?;
            app.css_size = (css_w, css_h);
            app.dpr = dpr;
        }
        let raw = translate::resize(f64::from(css_w), f64::from(css_h), dpr, t_ms as u64);
        apply_raw(app, raw)
    })
}
