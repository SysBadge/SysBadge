#[path = "simulator/defmt_stderr.rs"]
mod defmt_stderr;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

use embedded_graphics_simulator::{
    sdl2::Keycode, BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent,
    Window,
};
use sysbadge::system::{Member, SystemUf2};
use sysbadge::Button;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display =
        SimulatorDisplay::<BinaryColor>::new(Size::new(uc8151::WIDTH, uc8151::HEIGHT));

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .max_fps(1)
        .build();

    let system = create_system();
    let mut sysbadge = sysbadge::Sysbadge::new_with_system(display, &system);

    sysbadge.draw().unwrap();

    let mut window = Window::new("Sysbadge Simulator", &output_settings);

    'running: loop {
        sysbadge.draw().expect("Failed to redraw screen");
        window.update(&sysbadge.display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Q => break 'running,
                    Keycode::B => {
                        println!("register b press");
                        sysbadge.press(Button::B)
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(())
}

fn create_system() -> SystemUf2 {
    let members = Box::new([
        Member::new_str("Myriad", "they/them"),
        Member::new_str("Tester T. Testington", ""),
    ]);

    SystemUf2::new_from_box("Example system", members)
}
