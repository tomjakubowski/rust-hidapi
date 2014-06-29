#![crate_id = "blink1"]
#![crate_type = "bin"]

extern crate hidapi;
extern crate libc;

use hidapi::HidDevice;

static BLINK1_VENDOR_NUM: libc::c_ushort = 0x27B8;
static BLINK1_PRODUCT_NUM: libc::c_ushort = 0x01ED;

pub fn main() {
    use std::io::timer::sleep;

    let devices = hidapi::enumerate(0x0, 0x0);

    println!("{:16} {:48} {:6} {:6} {:6} {:4}\n",
             "manufacturer", "product",
             "vendor", "prod", "usepg", "useid");

    for dev in devices.iter() {
        println!("{:16} {:48} 0x{:04x} 0x{:04x} 0x{:04x} 0x{:02x}",
                 dev.manufacturer_string, dev.product_string,
                 dev.vendor_id, dev.product_id, dev.usage_page, dev.usage);
    }

    // BEGIN highfalutin (bad) way of doing:
    // let blink1 = HidDevice::open(BLINK1_VENDOR_NUM, BLINK1_PRODUCT_NUM).unwrap();
    let blink1 = devices.iter().find(|&inf| {
        inf.vendor_id == BLINK1_VENDOR_NUM && inf.product_id == BLINK1_PRODUCT_NUM
    });

    if blink1.is_none() {
        println!("blink(1) not found");
        return;
    }

    let blink1 = HidDevice::open_from_info(blink1.unwrap()).unwrap();
    // END highfalutin

    // set color to cyan
    let buf = [1u8, 'c' as u8, 0, 255, 255, 0, 0, 0];
    blink1.send_feature_report(buf);

    sleep(100); // bleh

    // fade to red over some time
    let dt = 100;
    let buf = [1u8, 'c' as u8, 255, 0, 0, (dt >> 8), (dt & 0xff), 0];
    blink1.send_feature_report(buf);
}
