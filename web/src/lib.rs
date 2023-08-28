use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
use embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder;
use sysbadge::system::{Member, SystemUf2};
use sysbadge::Sysbadge;
use wasm_bindgen::prelude::*;
use web_sys::console;

// Wee allocator as global alloc
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console::log_1(&"Hello, world!".into());

    let document = web_sys::window().unwrap().document().unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .scale(1)
        .pixel_spacing(1)
        .build();
    let mut display: WebSimulatorDisplay<BinaryColor> = WebSimulatorDisplay::new(
        (sysbadge::WIDTH, sysbadge::HEIGHT),
        &output_settings,
        document.get_element_by_id("sysbadge-app").as_ref(),
    );

    use embedded_graphics::Drawable;
    embedded_graphics::text::Text::new(
        "foo",
        Point::new(0, 0),
        embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_6X10,
            BinaryColor::On,
        ),
    )
    .draw(&mut display)
    .unwrap();

    let system = create_system();
    let mut sysbadge = Sysbadge::new_with_system(display, &system);

    sysbadge.draw().unwrap();

    sysbadge.display.flush().unwrap();

    Ok(())
}

fn create_system() -> SystemUf2 {
    let members = Box::new([
        Member::new_str("Myriad", "they/them"),
        Member::new_str("Tester T. Testington", ""),
    ]);

    SystemUf2::new_from_box("Example system", members)
}
