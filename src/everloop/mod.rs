mod led;
use crate::bus::memory_map::*;
use crate::Bus;
pub use led::Rgbw;
use core::intrinsics::transmute;
use heapless::{Vec, consts::U64 as MAX_LEDS};

/// Controls the ring of LEDS on a MATRIX device.
#[derive(Debug)]
pub struct Everloop<'a> {
    bus: &'a Bus,
}

impl<'a> Everloop<'a> {
    /// Return an instance of Everloop.
    pub fn new(bus: &Bus) -> Everloop {
        Everloop { bus }
    }

    /// Map each `RGBW` to the respective MATRIX LED. LEDs not set are defaulted to black.
    ///
    /// # Example
    /// ```
    /// # let bus = matrix_rhal::Bus::init().unwrap();
    /// let everloop = matrix_rhal::Everloop::new(&bus);
    /// // Set 15 LEDs to blue and the remaining to black
    /// everloop.set(&vec![matrix_rhal::Rgbw::new(0,0,255,0); 15]);
    /// ```
    pub fn set(&self, leds: &[Rgbw]) {
        let device_leds = self.bus.device_leds();
        if leds.len() > device_leds as usize {
            panic!(
                "Invalid LED set. This device only has {} LEDs",
                device_leds
            );
        }

        // create write buffer
        let capacity = device_leds as usize + 2;
        let mut request: Vec<i32, MAX_LEDS> = Vec::new();
        request.push(fpga_address::EVERLOOP as i32);
        request.push((device_leds * 4) as i32); // each LED RGBW requires 4 bytes

        // store all LED colors given
        for led in leds {
            request.push(led.as_bytes());
        }

        // set remaining LEDs to black
        for _ in 0..(capacity - request.len()) {
            request.push(Rgbw::black().as_bytes());
        }

        // render LEDs
        self.bus
            .write(unsafe { transmute::<&mut Vec<i32, MAX_LEDS>, &mut Vec<u8, MAX_LEDS>>(&mut request) });
    }

    /// Set all MATRIX LEDs to a single color
    pub fn set_all(&self, color: Rgbw) {
        let mut leds: Vec<Rgbw, MAX_LEDS> = Vec::new();
        leds.extend(core::iter::repeat(color).take(self.bus.device_leds() as usize));

        self.set(&leds)
    }
}
