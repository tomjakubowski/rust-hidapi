#![crate_type = "bin"]
#![crate_id = "x360"]

extern crate hidapi;

use hidapi::HidDevice;

// Requires a HID driver for the xbox 360 controller.

pub fn main() {
    use std::io::timer::sleep;

    let dev = HidDevice::open(0x045e, 0x028e).expect("No HID controller detected");
    println!("Found device! Enabling blinkenlights...");

    let buf = &[0x01, 0x03, 0x01]; // blink all lights
    dev.write(buf);

    sleep(2000);

    let buf = &[0x01, 0x03, 0x0A]; // 'rotate' lights
    dev.write(buf);

    sleep(2000);

    let buf = &[0x01, 0x03, 0x00]; // turn off lights
    dev.write(buf);
}
