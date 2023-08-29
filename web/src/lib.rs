use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use pkrs::model::System;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::sync::RwLock;
use sysbadge::system::{Member, SystemUf2};
use sysbadge::Sysbadge;
use wasm_bindgen::prelude::*;
use web_sys::{console, Document};

#[cfg(any(feature = "update", doc))]
pub mod update;

// Wee allocator as global alloc
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//static mut SYSTEM: SystemUf2 = SystemUf2::ZERO;
//static mut SYSBADGE: Option<Sysbadge<WebSimulatorDisplay<BinaryColor>>> = None;

#[wasm_bindgen(typescript_custom_section)]
const ISYSBADGE_CONF: &'static str = r#"
interface SysbadgeConfig {
    display?: String;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SysbadgeConfig")]
    pub type SysbadgeConfig;
}

#[wasm_bindgen]
pub struct SysbadgeApp {
    badge: Rc<RwLock<Sysbadge<'static, WebSimulatorDisplay<BinaryColor>>>>,
    //system: *mut SystemUf2,
    system: UnsafeCell<SystemUf2>,
}

#[wasm_bindgen]
impl SysbadgeApp {
    #[wasm_bindgen(constructor)]
    pub fn new(cfg: Option<SysbadgeConfig>) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();

        let display = Self::create_display(&document).unwrap();
        let system = create_system();
        let system = UnsafeCell::new(system);
        let badge = Sysbadge::new_with_system(display, unsafe { &*system.get() });

        Self {
            badge: Rc::new(RwLock::new(badge)),
            system,
        }
    }

    pub fn register_buttons(&self, cfg: Option<SysbadgeConfig>) {
        let document = web_sys::window().unwrap().document().unwrap();
        self.add_button_event_listener("sysbadge-app-button-b", &document, sysbadge::Button::B);
        /*add_button_event_listener("sysbadge-app-button-a", &document, sysbadge::Button::A);
        add_button_event_listener("sysbadge-app-button-b", &document, sysbadge::Button::B);
        add_button_event_listener("sysbadge-app-button-c", &document, sysbadge::Button::C);
        add_button_event_listener("sysbadge-app-button-up", &document, sysbadge::Button::Up);
        add_button_event_listener(
            "sysbadge-app-button-down",
            &document,
            sysbadge::Button::Down,
        );*/
    }

    /// Register a button press.
    pub fn press_button(&self, button: sysbadge::Button) {
        self.badge.write().unwrap().press(button);
    }

    /// Draw the badge to the display.
    pub fn draw(&self) {
        let mut badge = self.badge.write().unwrap();
        badge.draw().unwrap();
        badge.display.flush().unwrap();
    }
}

impl SysbadgeApp {
    const DISPLAY_ID: &'static str = "sysbadge-app-canvas";
    fn create_display(document: &Document) -> Result<WebSimulatorDisplay<BinaryColor>, JsValue> {
        let output_settings =
            embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder::new()
                .scale(1)
                .pixel_spacing(1)
                .build();

        let display: WebSimulatorDisplay<BinaryColor> = WebSimulatorDisplay::new(
            (sysbadge::WIDTH, sysbadge::HEIGHT),
            &output_settings,
            document.get_element_by_id(Self::DISPLAY_ID).as_ref(),
        );
        Ok(display)
    }

    fn set_system(&mut self, system: SystemUf2) -> SystemUf2 {
        let system = UnsafeCell::new(system);
        unsafe { core::ptr::swap(self.system.get(), system.get()) };
        self.badge.write().unwrap().system = unsafe { &*self.system.get() };

        system.into_inner()
    }

    fn add_button_event_listener(&self, id: &str, document: &Document, button: sysbadge::Button) {
        if let Some(button_elem) = document.get_element_by_id(id) {
            let badge = self.badge.clone();
            let closure = Closure::wrap(Box::new(move || {
                let mut badge = badge.write().unwrap();
                badge.press(button);
                badge.draw().unwrap();
                badge.display.flush().unwrap();
                drop(badge);
            }) as Box<dyn FnMut()>);

            button_elem
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget();
        }
    }
}

/*
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console::log_1(&"Hello, world!".into());

    let document = web_sys::window().unwrap().document().unwrap();

    let output_settings =
        embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder::new()
            .scale(1)
            .pixel_spacing(1)
            .build();
    let mut display: WebSimulatorDisplay<BinaryColor> = WebSimulatorDisplay::new(
        (sysbadge::WIDTH, sysbadge::HEIGHT),
        &output_settings,
        document.get_element_by_id("sysbadge-app-canvas").as_ref(),
    );

    let system = create_system();
    unsafe {
        SYSTEM = system;
    }
    let mut sysbadge = Sysbadge::new_with_system(display, unsafe { &SYSTEM });

    sysbadge.draw().unwrap();

    sysbadge.display.flush().unwrap();

    add_button_event_listener("sysbadge-app-button-a", &document, sysbadge::Button::A);
    add_button_event_listener("sysbadge-app-button-b", &document, sysbadge::Button::B);
    add_button_event_listener("sysbadge-app-button-c", &document, sysbadge::Button::C);
    add_button_event_listener("sysbadge-app-button-up", &document, sysbadge::Button::Up);
    add_button_event_listener(
        "sysbadge-app-button-down",
        &document,
        sysbadge::Button::Down,
    );

    unsafe {
        SYSBADGE = Some(sysbadge);
    }

    #[cfg(feature = "update")]
    update::register()?;

    Ok(())
}



#[wasm_bindgen]
pub fn press_button(button: sysbadge::Button) {
    console::log_1(&"Pressed button".into());
    let sysbadge = unsafe { SYSBADGE.as_mut().expect("SYSBADGE is None") };
    sysbadge.press(button);
    sysbadge.draw().unwrap();
    sysbadge.display.flush().unwrap();
}*/

fn create_system() -> SystemUf2 {
    let members = Box::new([
        Member::new_str("Myriad", "they/them"),
        Member::new_str("Tester T. Testington", ""),
    ]);

    SystemUf2::new_from_box("Example system".to_string().into_boxed_str(), members)
}
