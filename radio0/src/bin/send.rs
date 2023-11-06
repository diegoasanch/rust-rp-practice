#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::convert::Infallible;

use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{PIN_19, PIN_22, PIN_25, SPI1};
use embassy_rp::spi::{self, Blocking, Spi};
use embassy_time::{Duration, Timer};
use embedded_nrf24l01::{Configuration, CrcMode, DataRate, StandbyMode, NRF24L01};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<4096> = Stack::new();

static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

// tye
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::Low);

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(led))));
    });

    info!("Configuring NRF24L01");

    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let ce = Output::new(p.PIN_22, Level::Low);
    let csn = Output::new(p.PIN_19, Level::Low);

    let config = spi::Config::default();
    let spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, config);
    let mut nrf24 = NRF24L01::new(ce, csn, spi).unwrap();

    // nrf24.set_channel(8).unwrap();
    nrf24.set_auto_retransmit(0, 0).unwrap();
    nrf24.set_rf(&DataRate::R2Mbps, 3).unwrap();
    nrf24
        .set_pipes_rx_enable(&[true, false, false, false, false, false])
        .unwrap();
    nrf24.set_auto_ack(&[false; 6]).unwrap();
    nrf24.set_crc(CrcMode::Disabled).unwrap();
    nrf24.set_tx_addr(&b"fnord"[..]).unwrap();

    info!("Configured NRF24L01");

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_task(nrf24))));
}

#[embassy_executor::task]
async fn core0_task(
    nrf24: StandbyMode<
        NRF24L01<
            Infallible,
            Output<'static, PIN_22>,
            Output<'static, PIN_19>,
            Spi<'static, SPI1, Blocking>,
        >,
    >,
) {
    info!("Booting core 0");
    let mut index: u8 = 0;
    let mut tx = nrf24.tx().unwrap();
    // let message = b"Hello from the send side! Cycle: ";

    loop {
        Timer::after(Duration::from_millis(1000)).await;

        if tx.can_send().unwrap() {
            tx.send(&[index]).unwrap();
            info!("Sent {}", index);
        } else {
            info!("Can't send");
        }

        index += 1;
    }
}

#[embassy_executor::task]
async fn core1_task(mut led: Output<'static, PIN_25>) {
    info!("Booting core 1");

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        led.set_high();
        Timer::after(Duration::from_millis(1000)).await;
        led.set_low();
    }
}
