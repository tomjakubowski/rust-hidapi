#![crate_id = "hidapi#0.1.0"]
#![crate_type = "lib"]
#![license = "MIT"]

/*!

A Rust library for communicating with USB Human Interface Devices
using [`hidapi`](https://github.com/signal11/hidapi).

For example, you can send and receive feature reports with a
hypothetical USB cut of beef:

```no_run
use hidapi::HidDevice;

static MEAT_VENDOR_NUM: u16 = 0xdead;
static MEAT_PRODUCT_NUM: u16 = 0xbeef;

let steak = HidDevice::open(MEAT_VENDOR_NUM, MEAT_PRODUCT_NUM)
              .expect("where's the beef?");
steak.send_feature_report(bytes!(0x0, "MOO"));

let mut doneness = [0x01, 0x00, 0x00, 0x00, 0x00];
steak.get_feature_report(doneness);
assert_eq!(doneness.as_slice(), bytes!(1, "RARE"));
```

You can also enumerate the various cuts of beef attached to your
computer to find one that's also a golf club:

```no_run
use hidapi::{HidDevice, HidDeviceInfo};

static MEAT_VENDOR_NUM: u16 = 0xdead;
static MEAT_PRODUCT_NUM: u16 = 0xbeef;
static SPORTS_USAGE_PAGE: u16 = 0x04;
static GOLF_CLUB_USAGE: u16 = 0x02;

let steaks = hidapi::enumerate(MEAT_VENDOR_NUM, MEAT_PRODUCT_NUM);
let golf_steak: &HidDeviceInfo = steaks.iter().find(|&inf| {
    inf.usage_page == SPORTS_USAGE_PAGE && inf.usage == GOLF_CLUB_USAGE
}).expect("no golf steak found");

let device = HidDevice::open_from_info(golf_steak).unwrap();
device.send_feature_report(bytes!(0x0, "FORE"));
```
 */

extern crate libc;
extern crate sync;

use libc::size_t;
use sync::one::{Once, ONCE_INIT};

static mut INIT: Once = ONCE_INIT;

#[inline(always)]
unsafe fn init() {
    INIT.doit(|| {
        ffi::hid_init();
    });
}

mod ffi {
    use libc::{c_char, c_int, c_uchar, c_ushort, c_void, size_t, wchar_t};

    pub type HidDevice = c_void;

    pub struct HidDeviceInfo {
        pub path: *c_char,
        pub vendor_id: c_ushort,
        pub product_id: c_ushort,
        pub serial_number: *wchar_t,
        pub release_number: c_ushort,
        pub manufacturer_string: *wchar_t,
        pub product_string: *wchar_t,
        pub usage_page: c_ushort,
        pub usage: c_ushort,
        pub interface_number: c_int,
        pub next: *mut HidDeviceInfo
    }

    #[link(name = "hidapi")]
    extern "C" {
        pub fn hid_init() -> c_int;
        pub fn hid_enumerate(vendor_id: c_ushort, product_id: c_ushort) -> *mut HidDeviceInfo;
        pub fn hid_free_enumeration(devs: *mut HidDeviceInfo);
        pub fn hid_open(vendor_id: c_ushort, product_id: c_ushort, serial_number: *wchar_t)
                        -> *mut HidDevice;
        pub fn hid_open_path(path: *c_char) -> *mut HidDevice;
        pub fn hid_write(device: *mut HidDevice, data: *c_uchar, len: size_t) -> c_int;
        pub fn hid_read(device: *mut HidDevice, data: *mut c_uchar, len: size_t) -> c_int;

        pub fn hid_send_feature_report(device: *mut HidDevice, data: *c_uchar, len: size_t)
                                       -> c_int;
        pub fn hid_get_feature_report(device: *mut HidDevice, data: *mut c_uchar, len: size_t)
                                      -> c_int;
        pub fn hid_close(device: *mut HidDevice);
    }
}

/// Opaque structure representing a connected HID device.
pub struct HidDevice {
    dev: *mut ffi::HidDevice
}

impl HidDevice {
    /// Attempts to open a HID device using a `vendor_id` and
    /// `product_id`. Returns `None` if a device was not found.
    // FIXME: allow open with serial number
    pub fn open(vendor_id: libc::c_ushort, product_id: libc::c_ushort) -> Option<HidDevice> {
        use std::ptr::RawPtr;

        unsafe { init(); }

        let ptr = unsafe { ffi::hid_open(vendor_id, product_id, RawPtr::null()) };
        if ptr.is_null() {
            None
        } else {
            Some(HidDevice { dev: ptr })
        }
    }

    /// Attempts to open a HID device using the structure returned by
    /// [`hidapi::enumerate`](fn.enumerate.html).
    pub fn open_from_info(info: &HidDeviceInfo) -> Option<HidDevice> {
        HidDevice::open_path(info.path.as_slice())
    }

    /// Attempts to open a HID device by its path name.
    pub fn open_path(path: &str) -> Option<HidDevice> {
        unsafe { init(); }

        let ptr = path.with_c_str(|c_str| {
            unsafe { ffi::hid_open_path(c_str) }
        });

        if ptr.is_null() {
            None
        } else {
            Some(HidDevice { dev: ptr })
        }
    }

    /// Sends a feature report to the device. The first byte of `buf`
    /// should be the report ID.
    // FIXME: figure out how to deal with errors
    // FIXME: should these all be &mut self?
    pub fn send_feature_report(&self, buf: &[u8]) {
        unsafe {
            ffi::hid_send_feature_report(self.dev, &buf[0], buf.len() as size_t);
        }
    }

    /// Reads a feature report from the device into `buf`. The first
    /// byte of `buf` should be set to the report ID to be read before
    /// calling this method.
    pub fn get_feature_report(&self, buf: &mut [libc::c_uchar]) {
        unsafe {
            let size = ffi::hid_get_feature_report(self.dev, &mut buf[0], buf.len() as size_t);

            if size == -1 {
                fail!("failed to read feature report");
            }
        };
    }

    /// Reads an input report from the device into `buf`.
    // FIXME: add support for hid_read_timeout
    pub fn read(&self, buf: &mut [libc::c_uchar]) {
        unsafe {
            let size = ffi::hid_read(self.dev, &mut buf[0], buf.len() as size_t);
            if size == -1 {
                fail!("failed to read");
            }
        }
    }

    /// Writes an output report to the device.
    pub fn write(&self, buf: &[libc::c_uchar]) {
        unsafe {
            ffi::hid_write(self.dev, &buf[0], buf.len() as size_t);
        }
    }
}

impl Drop for HidDevice {
    fn drop(&mut self) {
        unsafe {
            ffi::hid_close(self.dev);
        }
    }
}

#[deriving(Show)]
pub struct HidDeviceInfo {
    pub path: String,
    pub vendor_id: libc::c_ushort,
    pub product_id: libc::c_ushort,
    #[doc(hidden)]
    pub serial_number: String,
    pub release_number: libc::c_ushort,
    #[doc(hidden)]
    pub manufacturer_string: String,
    #[doc(hidden)]
    pub product_string: String,
    pub usage_page: libc::c_ushort,
    pub usage: libc::c_ushort,
    pub interface_number: libc::c_int
}

impl HidDeviceInfo {
    fn from_raw_device_info(dev: *ffi::HidDeviceInfo) -> HidDeviceInfo {
        unsafe {
            HidDeviceInfo {
                path: std::str::raw::from_c_str((*dev).path),
                vendor_id: (*dev).vendor_id,
                product_id: (*dev).product_id,
                serial_number: "FIXME".to_string(),
                release_number: (*dev).release_number,
                manufacturer_string: "FIXME".to_string(),
                product_string: "FIXME".to_string(),
                usage_page: (*dev).usage_page,
                usage: (*dev).usage,
                interface_number: (*dev).interface_number
            }
        }
    }
}

/// Returns a vector of the HID devices that match `vendor_id` and
/// `product_id`. If `vendor_id` is set to 0, any vendor matches. If
/// `product_id` is set to 0, any product matches. If both `vendor_id`
/// and `product_id` are set to 0, all HID devices are returned.
pub fn enumerate(vendor_id: libc::c_ushort, product_id: libc::c_ushort) -> Vec<HidDeviceInfo> {
    unsafe {
        init();

        let head = ffi::hid_enumerate(vendor_id, product_id);
        let mut devs = vec![];
        let mut cur = head;

        while !cur.is_null() {
            devs.push(HidDeviceInfo::from_raw_device_info(cur as *ffi::HidDeviceInfo));
            cur = (*cur).next;
        }
        ffi::hid_free_enumeration(head);
        devs
    }
}
