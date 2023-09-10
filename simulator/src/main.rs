mod defmt_stderr;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

use embedded_graphics_simulator::{
    sdl2::Keycode, BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent,
    Window,
};
use sysbadge::system::{MemberUF2, SystemUf2, SystemVec};
use sysbadge::{Button, Sysbadge};

fn main() -> Result<(), core::convert::Infallible> {
    let mut display =
        SimulatorDisplay::<BinaryColor>::new(Size::new(sysbadge::WIDTH, sysbadge::HEIGHT));

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .max_fps(1)
        .build();

    let system = create_system();
    let mut sysbadge = Sysbadge::new_with_system(display, system);

    sysbadge.draw().unwrap();

    let window = Window::new("Sysbadge Simulator", &output_settings);

    let args: Vec<Button> = std::env::args()
        .skip(1)
        .map(|s| convert_to_button(&s))
        .collect();

    if args.is_empty() {
        run_loop(window, sysbadge);
    } else {
        run_buttons(window, sysbadge, args);
    }

    Ok(())
}

fn run_loop(mut window: Window, mut sysbadge: Sysbadge<SimulatorDisplay<BinaryColor>, SystemVec>) {
    'running: loop {
        sysbadge.draw().expect("Failed to redraw screen");
        window.update(&sysbadge.display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::Q => break 'running,
                    Keycode::Y => {
                        println!("register user press");
                        sysbadge.press(Button::USER)
                    }
                    Keycode::X => {
                        println!("register a press");
                        sysbadge.press(Button::A)
                    }
                    Keycode::C => {
                        println!("register b press");
                        sysbadge.press(Button::B)
                    }
                    Keycode::V => {
                        println!("register c press");
                        sysbadge.press(Button::C)
                    }
                    Keycode::Up | Keycode::B => {
                        println!("register up press");
                        sysbadge.press(Button::Up)
                    }
                    Keycode::Down | Keycode::N => {
                        println!("register down press");
                        sysbadge.press(Button::Down)
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn run_buttons(
    mut window: Window,
    mut sysbadge: Sysbadge<SimulatorDisplay<BinaryColor>, SystemVec>,
    buttons: Vec<Button>,
) {
    for button in buttons {
        sysbadge.press(button);
    }

    sysbadge.draw().expect("Failed to redraw screen");
    window.update(&sysbadge.display);

    'running: loop {
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
    }
}

fn convert_to_button(str: &str) -> Button {
    match str.to_lowercase().as_str() {
        "a" => Button::A,
        "b" => Button::B,
        "c" => Button::C,
        "up" => Button::Up,
        "down" => Button::Down,
        "user" => Button::USER,
        _ => panic!("Unknown button {}", str),
    }
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
