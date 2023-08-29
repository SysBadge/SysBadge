use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use sysbadge::system::{Member, SystemUf2};
use sysbadge::Sysbadge;
use wasm_bindgen::prelude::*;
use web_sys::{console, Document};
static mut SYSTEM: SystemUf2 = SystemUf2::ZERO;
static mut SYSBADGE: Option<Sysbadge<WebSimulatorDisplay<BinaryColor>>> = None;

pub(crate) fn register(document: &Document) -> Result<(), JsValue> {
    if let Some(app) = document.get_element_by_id("sysbadge-badge") {
        app.set_inner_html(include_str!("badge.html"));

        let output_settings =
            embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder::new()
                .scale(1)
                .pixel_spacing(1)
                .build();
        let mut display: WebSimulatorDisplay<BinaryColor> = WebSimulatorDisplay::new(
            (sysbadge::WIDTH, sysbadge::HEIGHT),
            &output_settings,
            document
                .get_element_by_id("_sysbadge-badge-canvas")
                .as_ref(),
        );

        let system = create_system();
        unsafe {
            SYSTEM = system;
        }
        let mut sysbadge = Sysbadge::new_with_system(display, unsafe { &SYSTEM });

        sysbadge.draw().unwrap();

        sysbadge.display.flush().unwrap();

        add_button_event_listener("_sysbadge-badge-button-a", &document, sysbadge::Button::A);
        add_button_event_listener("_sysbadge-badge-button-b", &document, sysbadge::Button::B);
        add_button_event_listener("_sysbadge-badge-button-c", &document, sysbadge::Button::C);
        add_button_event_listener("_sysbadge-badge-button-up", &document, sysbadge::Button::Up);
        add_button_event_listener(
            "_sysbadge-badge-button-down",
            &document,
            sysbadge::Button::Down,
        );

        unsafe {
            SYSBADGE = Some(sysbadge);
        }
    }

    Ok(())
}

fn add_button_event_listener(id: &str, document: &Document, button: sysbadge::Button) {
    if let Some(button_elem) = document.get_element_by_id(id) {
        let closure = Closure::wrap(Box::new(move || {
            press_button(button);
        }) as Box<dyn FnMut()>);

        button_elem
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }
}

#[wasm_bindgen]
pub fn press_button(button: sysbadge::Button) {
    console::log_1(&"Pressed button".into());
    let sysbadge = unsafe { SYSBADGE.as_mut().expect("SYSBADGE is None") };
    sysbadge.press(button);
    sysbadge.draw().unwrap();
    sysbadge.display.flush().unwrap();
}

fn create_system() -> SystemUf2 {
    let members = Box::new([
        Member::new_str("Myriad", "they/them"),
        Member::new_str("Tester T. Testington", ""),
    ]);

    SystemUf2::new_from_box("Example system".to_string().into_boxed_str(), members)
}
