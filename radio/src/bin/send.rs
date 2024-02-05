#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Level, Output},
    spi::{self, Spi},
};
use embassy_time::{Duration, Timer};
use embedded_nrf24l01::{Configuration, CrcMode, DataRate, NRF24L01};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    info!("Configuring NRF24L01");

    let miso = p.PIN_4;
    let mosi = p.PIN_7;
    let sck = p.PIN_6;
    let ce = Output::new(p.PIN_17, Level::Low);
    let csn = Output::new(p.PIN_14, Level::Low);

    let config = spi::Config::default();
    let spi = Spi::new_blocking(p.SPI0, sck, mosi, miso, config);
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
    nrf24.set_rx_addr(0, &b"fnord"[..]).unwrap();

    let mut radio = nrf24.tx().unwrap();

    loop {
        match radio.can_send() {
            Ok(true) => match radio.send("Hello".as_bytes()) {
                Ok(_) => {
                    info!("Sent");
                }
                Err(_) => {
                    info!("Error: failed to send");
                    continue;
                }
            },
            Ok(false) => {}
            Err(_) => {
                info!("Error: cannot send");
                continue;
            }
        }

        // info!("Sending On");
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
        // info!("Sending Off");
        Timer::after(Duration::from_millis(900)).await;
    }
}
