use std::thread::sleep_ms;
use sysbadge::Button;
use sysbadge_usb::UsbSysbadge;

fn main() {
    let context = rusb::Context::new().unwrap();
    let mut usb = UsbSysbadge::open(context).unwrap();

    usb.press(Button::C).unwrap();
    println!("System name: {}", usb.system_name().unwrap());
    println!("Member count: {}", usb.member_count().unwrap());
    println!("Member name: {}", usb.member_name(0).unwrap());
    println!("Member pronouns: {}", usb.member_pronouns(0).unwrap());

    let mut state = usb.get_state().unwrap();
    state.change(Button::C, 2);
    usb.set_state(&state).unwrap();
}
