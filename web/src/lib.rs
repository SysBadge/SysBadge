use pkrs::model::System;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;
use web_sys::{console, Document};

mod badge;
#[cfg(any(feature = "update", doc))]
pub mod update;

// Wee allocator as global alloc
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();

    #[cfg(feature = "badge")]
    badge::register(&document)?;

    #[cfg(feature = "update")]
    update::register()?;

    Ok(())
}
