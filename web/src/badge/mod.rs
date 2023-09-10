use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use sysbadge::badge::Sysbadge;
use sysbadge::system::{MemberUF2, SystemUf2, SystemVec};
use wasm_bindgen::prelude::*;
use web_sys::{console, Document};

pub(crate) static mut SYSBADGE: Option<Sysbadge<WebSimulatorDisplay<BinaryColor>, SystemVec>> =
    None;

pub(crate) fn register(document: &Document) -> Result<(), JsValue> {
    if let Some(app) = document.get_element_by_id("sysbadge-badge") {
        app.set_inner_html(include_str!("badge.html"));

        let output_settings =
            embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder::new()
                .scale(1)
                .pixel_spacing(1)
                .build();
        let display: WebSimulatorDisplay<BinaryColor> = WebSimulatorDisplay::new(
            (sysbadge::WIDTH, sysbadge::HEIGHT),
            &output_settings,
            document
                .get_element_by_id("_sysbadge-badge-canvas")
                .as_ref(),
        );

        let system = create_system();
        let mut sysbadge = Sysbadge::new_with_system(display, system);

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

        {
            let closure = Closure::wrap(Box::new(move || unsafe {
                let badge = SYSBADGE.as_mut().unwrap();
                badge.reset();
                badge.draw().unwrap();
                badge.display.flush().unwrap();
            }) as Box<dyn FnMut()>);

            document
                .get_element_by_id("_sysbadge-badge-button-reset")
                .unwrap()
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();

            closure.forget();
        }

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

fn create_system() -> SystemVec {
    let mut system = SystemVec::new("PluralKit Example System".to_string());
    system.members.push(sysbadge::system::MemberStrings {
        name: "Myriad".to_string(),
        pronouns: "they/them".to_string(),
    });
    system.members.push(sysbadge::system::MemberStrings {
        name: "Tester T. Testington".to_string(),
        pronouns: "".to_string(),
    });
    system
}
