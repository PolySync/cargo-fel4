#![no_std]
extern crate sel4_sys;

#[cfg(test)]
mod fel4_test;

// TODO - feature-flag the debugging resources
use sel4_sys::DebugOutHandle;

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        DebugOutHandle.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn run() {
    println!("\nhello from a fel4 app!\n");
}
