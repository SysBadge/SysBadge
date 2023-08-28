use defmt::Formatter;
use std::io::{stderr, Write};
use std::sync::atomic::{AtomicBool, Ordering};

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because interrupts are disabled
        TAKEN.store(true, Ordering::Relaxed);
    }

    unsafe fn release() {
        TAKEN.store(false, Ordering::Relaxed);
    }

    unsafe fn write(bytes: &[u8]) {
        stderr().write_all(bytes).ok();
    }

    unsafe fn flush() {
        stderr().flush().ok();
    }
}

#[export_name = "_defmt_panic"]
fn defmt_panic(info: &core::panic::PanicInfo) -> ! {
    core::panic!("{}", info);
}

#[export_name = "_defmt_timestamp"]
fn defmt_timestamp(_f: Formatter<'_>) {}
