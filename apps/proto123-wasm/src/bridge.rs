//! wasm-bindgen exports and the Ratzilla canvas. Ratzilla is used for drawing only: all input
//! arrives as raw primitives from `web/host.js` and is interpreted by the shared core.

use crate::translate;
use proto123_engine::{Command, Redraw, Session};
use proto123_theme::high_contrast;
use proto123_tui::{CellMetrics, InputMapper, LayoutMap, RawKind, render};
use ratatui::Terminal;
use ratzilla::backend::canvas::{CanvasBackend, CanvasBackendOptions};
use ratzilla::backend::cell_sized::CellSized;
use ratzilla::web_sys;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

struct App {
    parent_id: String,
    session: Session,
    mapper: InputMapper,
    layout: LayoutMap,
    terminal: Terminal<CanvasBackend>,
    metrics: CellMetrics,
    grid_cells: (u16, u16),
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

fn make_terminal(
    parent_id: &str,
    css: (u32, u32),
    dpr: f64,
    theme_metrics: &proto123_theme::GridMetrics,
) -> Result<(Terminal<CanvasBackend>, CellMetrics, bool), JsValue> {
    remove_previous_canvas(parent_id)?;
    let px_w = (f64::from(css.0) * dpr).round() as u32;
    let px_h = (f64::from(css.1) * dpr).round() as u32;
    let options = CanvasBackendOptions::new()
        .grid_id(parent_id)
        .size((px_w, px_h));
    let backend = CanvasBackend::new_with_options(options).map_err(js_err)?;
    let (bw, bh) = backend.cell_size_css_px();
    let (metrics, mismatch) =
        CellMetrics::from_theme(theme_metrics, dpr).reconcile((f64::from(bw), f64::from(bh)));
    let terminal = Terminal::new(backend).map_err(js_err)?;
    Ok((terminal, metrics, mismatch))
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

fn apply_raw(app: &mut App, event: proto123_tui::RawEvent) -> Result<Option<String>, JsValue> {
    let mut redraw = Redraw::None;
    let mut announce = None;
    for command in app.mapper.map(event, &app.layout, &app.session) {
        let out = app.session.dispatch(command);
        redraw = combine(redraw, out.redraw);
        if out.announce.is_some() {
            announce = out.announce;
        }
    }
    match redraw {
        Redraw::None => {}
        Redraw::Regions => draw(app, false)?,
        Redraw::Full => draw(app, true)?,
    }
    Ok(announce)
}

fn apply_resize(
    app: &mut App,
    css_w: u32,
    css_h: u32,
    mismatch: bool,
    t_ms: u64,
) -> Result<Option<String>, JsValue> {
    let raw =
        translate::resize_with_metrics(&app.metrics, f64::from(css_w), f64::from(css_h), t_ms);
    if let RawKind::Resize { cols, rows } = raw.kind {
        app.grid_cells = (cols, rows);
    }
    if mismatch {
        let mut cmds = app.mapper.map(raw, &app.layout, &app.session);
        let mut redraw = Redraw::None;
        let mut announce = None;
        for cmd in cmds.drain(..) {
            let mapped = match cmd {
                Command::SessionResize { cols, rows } => Command::SessionResizeWithMetrics {
                    cols,
                    rows,
                    metrics_mismatch: true,
                },
                other => other,
            };
            let out = app.session.dispatch(mapped);
            redraw = combine(redraw, out.redraw);
            if out.announce.is_some() {
                announce = out.announce;
            }
        }
        if redraw == Redraw::Full {
            draw(app, true)?;
        } else if redraw == Redraw::Regions {
            draw(app, false)?;
        }
        return Ok(announce);
    }
    apply_raw(app, raw)
}

/// Starts the app inside `parent_id` at the given CSS size and device pixel ratio.
#[wasm_bindgen]
pub fn start(parent_id: &str, css_w: u32, css_h: u32, dpr: f64) -> Result<Option<String>, JsValue> {
    let session = Session::new(high_contrast());
    let theme_metrics = session.theme().metrics;
    let (terminal, metrics, mismatch) =
        make_terminal(parent_id, (css_w, css_h), dpr, &theme_metrics)?;
    let mut app = App {
        parent_id: parent_id.to_owned(),
        session,
        mapper: InputMapper::new(theme_metrics),
        layout: LayoutMap::default(),
        terminal,
        metrics,
        grid_cells: (0, 0),
        css_size: (css_w, css_h),
        dpr,
    };
    let announce = apply_resize(&mut app, css_w, css_h, mismatch, 0)?;
    APP.with(|cell| *cell.borrow_mut() = Some(app));
    Ok(announce)
}

/// Forwards one `keydown` and returns any pending live-region announcement.
#[wasm_bindgen]
pub fn on_key(
    key: &str,
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
    t_ms: f64,
) -> Result<Option<String>, JsValue> {
    let Some(raw) = translate::keydown(key, shift, ctrl, alt, meta, t_ms as u64) else {
        return Ok(None);
    };
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => apply_raw(app, raw),
        None => Ok(None),
    })
}

/// Forwards a pointer event (`0` down, `1` move, `2` up, `3` leave) in CSS px.
#[wasm_bindgen]
pub fn on_pointer(
    kind: u8,
    x: f64,
    y: f64,
    button: i16,
    buttons: u16,
    mods: u8,
    t_ms: f64,
) -> Result<Option<String>, JsValue> {
    APP.with(|cell| {
        let mut slot = cell.borrow_mut();
        let Some(app) = slot.as_mut() else {
            return Ok(None);
        };
        let (cols, rows) = app.grid_cells;
        let Some(raw) = translate::pointer(
            translate::DomPointer {
                kind,
                x,
                y,
                button,
                buttons,
                mods_bits: mods,
            },
            t_ms as u64,
            &app.metrics,
            cols,
            rows,
        ) else {
            return Ok(None);
        };
        apply_raw(app, raw)
    })
}

/// Forwards a wheel event (`dx`, `dy`) and returns any pending announcement.
#[wasm_bindgen]
pub fn on_wheel(dx: f64, dy: f64, mods: u8, t_ms: f64) -> Result<Option<String>, JsValue> {
    let Some(raw) = translate::wheel(dx, dy, mods, t_ms as u64) else {
        return Ok(None);
    };
    APP.with(|cell| match cell.borrow_mut().as_mut() {
        Some(app) => apply_raw(app, raw),
        None => Ok(None),
    })
}

/// Forwards a canvas size or device-pixel-ratio change; rebuilds the canvas when it changed.
#[wasm_bindgen]
pub fn on_resize(css_w: u32, css_h: u32, dpr: f64, t_ms: f64) -> Result<Option<String>, JsValue> {
    APP.with(|cell| {
        let mut slot = cell.borrow_mut();
        let Some(app) = slot.as_mut() else {
            return Ok(None);
        };
        let mut mismatch = false;
        if app.css_size != (css_w, css_h) || (app.dpr - dpr).abs() > f64::EPSILON {
            let theme_metrics = app.session.theme().metrics;
            let (terminal, metrics, mm) =
                make_terminal(&app.parent_id, (css_w, css_h), dpr, &theme_metrics)?;
            app.terminal = terminal;
            app.metrics = metrics;
            app.css_size = (css_w, css_h);
            app.dpr = dpr;
            mismatch = mm;
        }
        apply_resize(app, css_w, css_h, mismatch, t_ms as u64)
    })
}
