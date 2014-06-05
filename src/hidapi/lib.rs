#![crate_id = "hidapi#0.1.0"]
#![crate_type = "lib"]
#![license = "MIT"]

extern crate libc;

use std::rt::libc_heap::malloc_raw;

static MAX_USB_STRBUF_SIZE: uint = 127;

// FIXME: this is most certainly broken: no locale set, probably
// doesn't work on windows, etc.
fn from_wstring(wstr: *libc::wchar_t) -> String {
    use std::mem::size_of;
    use libc::{c_char, c_void, size_t};

    let c_str = unsafe {
        let bufsize = MAX_USB_STRBUF_SIZE;
        let buf = malloc_raw(bufsize * size_of::<c_char>()) as *mut c_char;

        let bytes = ffi::wcstombs(buf, wstr, bufsize as size_t);
        if bytes == bufsize as size_t {
            let term = buf.offset(bytes as int - 1);
            *term = '\0' as i8;
        }
        buf
    };

    unsafe {
        let ret = std::str::raw::from_c_str(c_str as *c_char);
        libc::free(c_str as *mut c_void);
        ret
    }
}

#[allow(dead_code, raw_pointer_deriving)]
mod ffi {
    use libc::{c_char, c_int, c_uchar, c_ushort, c_void, size_t, wchar_t};

    pub type HidDevice = c_void;

    #[deriving(Show)]
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
        pub fn hid_exit() -> c_int;
        pub fn hid_enumerate(vendor_id: c_ushort, product_id: c_ushort) -> *mut HidDeviceInfo;
        pub fn hid_free_enumeration(devs: *mut HidDeviceInfo);
        pub fn hid_open(vendor_id: c_ushort, product_id: c_ushort, serial_number: *wchar_t)
                        -> *mut HidDevice;
        pub fn hid_open_path(path: *c_char) -> *mut HidDevice;
        pub fn hid_close(device: *mut HidDevice);
        pub fn hid_send_feature_report(device: *mut HidDevice, data: *c_uchar, len: size_t)
                                       -> c_int;
        pub fn hid_get_feature_report(device: *mut HidDevice, data: *mut c_uchar, len: size_t)
                                      -> c_int;
        pub fn hid_get_product_string(device: *mut HidDevice, string: *mut wchar_t, maxlen: size_t)
                                      -> c_int;
    }

    extern "C" {
        pub fn wcstombs(dst: *mut c_char, src: *wchar_t, len: size_t) -> size_t;
    }
}

pub struct HidDevice {
    dev: *mut ffi::HidDevice
}

impl HidDevice {
    pub fn open(vendor_id: libc::c_ushort, product_id: libc::c_ushort) -> Option<HidDevice> {
        use std::ptr::RawPtr;

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
        let ptr = path.with_c_str(|c_str| {
            unsafe { ffi::hid_open_path(c_str) }
        });

        if ptr.is_null() {
            None
        } else {
            Some(HidDevice { dev: ptr })
        }
    }

    // TODO: error handling?
    pub fn send_feature_report(&self, buf: &[u8]) {
        unsafe {
            ffi::hid_send_feature_report(self.dev, &buf[0], buf.len() as libc::size_t);
        }
    }

    pub fn get_feature_report(&self, buf: &mut [libc::c_uchar]) {
        use libc::size_t;

        unsafe {
            let size = ffi::hid_get_feature_report(self.dev, &mut buf[0], buf.len() as size_t);

            if size == -1 {
                fail!("failed to read feature report");
            }
        };
    }

    pub fn get_product_string(&self) -> String {
        use std::mem::size_of;

        unsafe {
            let bufsize = MAX_USB_STRBUF_SIZE;
            let buf = (bufsize * size_of::<libc::wchar_t>()) as *mut _;

            let err = ffi::hid_get_product_string(self.dev, buf, bufsize as libc::size_t);
            if err != 0 {
                fail!("failed to get product string");
            }

            let ret = from_wstring(buf as *libc::wchar_t);
            libc::free(buf as *mut libc::c_void);
            ret
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
                serial_number: from_wstring((*dev).serial_number),
                release_number: (*dev).release_number,
                manufacturer_string: from_wstring((*dev).manufacturer_string),
                product_string: from_wstring((*dev).product_string),
                usage_page: (*dev).usage_page,
                usage: (*dev).usage,
                interface_number: (*dev).interface_number
            }
        }
    }
}

pub fn enumerate(vendor_id: libc::c_ushort, product_id: libc::c_ushort) -> Vec<HidDeviceInfo> {
    unsafe {
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
