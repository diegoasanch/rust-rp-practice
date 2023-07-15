#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{PIN_15, PIN_25};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();

static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
// Channel for communication between cores.
static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

enum LedState {
    On,
    Off,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::Low);
    let button = Input::new(p.PIN_15, Pull::Down);

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(led))));
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(log_cycle()));
        unwrap!(spawner.spawn(core0_task(button)));
    });
}

#[embassy_executor::task]
async fn core0_task(mut button: Input<'static, PIN_15>) {
    info!("Booting core 0");

    let mut current_state = LedState::Off;
    let mut last_pressed = Instant::now();
    let mut now;

    loop {
        button.wait_for_any_edge().await;
        button.wait_for_high().await;

        now = Instant::now();
        if now - last_pressed < Duration::from_millis(300) {
            continue;
        }
        last_pressed = now;

        info!("Button pressed");

        match current_state {
            LedState::On => {
                current_state = LedState::Off;
                CHANNEL.send(LedState::Off).await;
            }
            LedState::Off => {
                current_state = LedState::On;
                CHANNEL.send(LedState::On).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn core1_task(mut led: Output<'static, PIN_25>) {
    info!("Booting core 1");
    let mut cycle = 0;

    loop {
        match CHANNEL.recv().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
        }
        println!("Blink ;) {}", cycle);
        cycle += 1;
    }
}

#[embassy_executor::task]
async fn log_cycle() {
    let mut cycle = 0;
    loop {
        println!("Cycle {}", cycle);
        cycle += 1;
        Timer::after(Duration::from_millis(1000)).await;
    }
}
