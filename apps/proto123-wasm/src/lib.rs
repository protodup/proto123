//! Web front end. `translate` is target-neutral (DOM-shaped primitives in, `RawEvent` out)
//! and is tested natively; `bridge` holds the wasm-bindgen exports and the Ratzilla canvas.

pub mod translate;

#[cfg(target_arch = "wasm32")]
mod bridge;
