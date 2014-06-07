#![crate_id = "hidapi#0.1.0"]
#![crate_type = "lib"]
#![license = "MIT"]

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

pub struct HidDevice {
    dev: *mut ffi::HidDevice
}

impl HidDevice {
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

    pub fn open_from_info(info: &HidDeviceInfo) -> Option<HidDevice> {
        HidDevice::open_path(info.path.as_slice())
    }

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

    // FIXME: figure out how to deal with errors
    // FIXME: should these all be &mut self?
    pub fn send_feature_report(&self, buf: &[u8]) {
        unsafe {
            ffi::hid_send_feature_report(self.dev, &buf[0], buf.len() as size_t);
        }
    }

    pub fn get_feature_report(&self, buf: &mut [libc::c_uchar]) {
        unsafe {
            let size = ffi::hid_get_feature_report(self.dev, &mut buf[0], buf.len() as size_t);

            if size == -1 {
                fail!("failed to read feature report");
            }
        };
    }

    // FIXME: add support for hid_read_timeout
    pub fn read(&self, buf: &mut [libc::c_uchar]) {
        unsafe {
            let size = ffi::hid_read(self.dev, &mut buf[0], buf.len() as size_t);
            if size == -1 {
                fail!("failed to read");
            }
        }
    }

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
    pub serial_number: String,
    pub release_number: libc::c_ushort,
    pub manufacturer_string: String,
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
