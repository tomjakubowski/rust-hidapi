#![crate_type = "bin"]
#![crate_id = "x360"]
#![feature(macro_rules)]

extern crate hidapi;

use hidapi::HidDevice;

use std::fmt;

// Requires a HID driver for the xbox 360 controller.

#[deriving(PartialEq, Eq)]
#[packed]
struct Xbox360Msg {
    kind: u8,
    length: u8,
    buttons: u16,
    trig_l: u8,
    trig_r: u8,
    thumb_l_x: i16,
    thumb_l_y: i16,
    thumb_r_x: i16,
    thumb_r_y: i16,
    dummy2_: u32,
    dummy3_: u16
}

macro_rules! define_buttons_trait(
    ($($fun:ident),*) => (
        trait Xbox360Buttons {
            $(fn $fun(&self) -> bool;)*
        }
    )
)

define_buttons_trait!(dpad_up, dpad_down, dpad_left, dpad_right, start, back,
                      thumb_l, thumb_r, shoulder_l, shoulder_r, guide,
                      a, b, x, y)

macro_rules! impl_buttons(
    ($strukt:ident, $($but:ident: $idx:expr),*) => (
        impl Xbox360Buttons for $strukt {
            $(
                #[inline(always)]
                fn $but(&self) -> bool {
                    self.buttons & (1 << $idx) != 0
                }
            )*
        }
    )
)

impl Xbox360Msg {
    fn new(buf: [u8, ..20]) -> Xbox360Msg{
        use std::mem;

        unsafe {
            mem::transmute(buf)
        }
    }
}

impl fmt::Show for Xbox360Msg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = if self.a() { 'A' } else { ' ' };
        let b = if self.b() { 'B' } else { ' ' };
        let x = if self.x() { 'X' } else { ' ' };
        let y = if self.y() { 'Y' } else { ' ' };
        let back = if self.back() { '<' } else { ' ' };
        let start = if self.start() { '>' } else { ' ' };
        let guide = if self.guide() { 'G' } else { ' ' };

        let dl = if self.dpad_left() { 'L' } else { ' ' };
        let dr = if self.dpad_right() { 'R' } else { ' ' };
        let du = if self.dpad_up() { 'U' } else { ' ' };
        let dd = if self.dpad_down() { 'D' } else { ' ' };


        let lb = if self.shoulder_l() { "LB" } else { "  " };
        let rb = if self.shoulder_r() { "RB" } else { "  " };

        write!(f, concat!("thumb_l: ({thl_x:+6d},{thl_y:+6d}) ",
                          "thumb_r: ({thr_x:+6d},{thr_y:+6d}) ",
                          "trigs: {trig_l:2x} {trig_r:2x} ",
                          "[{dl}{du}{dr}{dd}]",
                          "[{lb} {rb}]",
                          "[{a}{b}{x}{y}{back}{start}{guide}]"),
               thl_x=self.thumb_l_x, thl_y=self.thumb_l_y,
               thr_x=self.thumb_r_x, thr_y=self.thumb_r_y,
               trig_l=self.trig_l, trig_r=self.trig_r,
               dl=dl, dr=dr, du=du, dd=dd,
               lb=lb, rb=rb,
               a=a, b=b, x=x, y=y, back=back, start=start, guide=guide)
    }
}

impl_buttons!(Xbox360Msg,
              dpad_up: 0,
              dpad_down: 1,
              dpad_left: 2,
              dpad_right: 3,
              start: 4,
              back: 5,
              thumb_l: 6,
              thumb_r: 7,
              shoulder_l: 8,
              shoulder_r: 9,
              guide: 10,
              // dummy
              a: 12,
              b: 13,
              x: 14,
              y: 15)

pub fn main() {
    use std::mem::size_of;

    let dev = HidDevice::open(0x045e, 0x028e).expect("No HID controller detected");
    println!("Found device! Enabling blinkenlights...");
    println!("size: {}", size_of::<bool>());

    loop {
        let msg = {
            let mut buf = [0u8, ..20];
            dev.read(buf);
            Xbox360Msg::new(buf)
        };
        println!("{}", msg);
        // println!("{} {} {} {}", msg.a(), msg.b(), msg.x(), msg.y());
        // println!("buf {:08t} {:08t}", buf[2], buf[3]);
    }
}
