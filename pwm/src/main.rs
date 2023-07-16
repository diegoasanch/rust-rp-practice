//! This example shows how to use PWM (Pulse Width Modulation) in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. Add a LED and resistor to another pin.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm as RpPwm};
use embassy_time::{Duration, Timer};
use pwm::{Pwm, PwmSlice};
use {defmt_rtt as _, panic_probe as _};

mod pwm;

enum Direction {
    Up,
    Down,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut c: Config = Default::default();
    c.top = 256;
    c.compare_b = 8;

    let pwm = RpPwm::new_output_b(p.PWM_CH4, p.PIN_25, c.clone());
    let mut led = Pwm::new(pwm, c, PwmSlice::B);

    let mut duty: u16 = 0;
    let mut direction = Direction::Up;
    let step = 10;

    loop {
        Timer::after(Duration::from_millis(25)).await;

        if duty >= led.get_max_duty() {
            direction = Direction::Down;
        } else if duty == 0 {
            direction = Direction::Up;
        }

        match direction {
            Direction::Up => duty += step,
            Direction::Down => duty -= step,
        }

        led.set_duty(duty);
    }
}
