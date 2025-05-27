#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use defmt_rtt as _;
use defmt::info;
use stm32f0xx_hal::{
    adc::{Adc, VTemp},
    delay::Delay,
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    // Initialize RTT
    info!("Starting program...");

    // Get access to cortex-m peripherals
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Get access to the device peripherals
    let mut dp = pac::Peripherals::take().unwrap();
    info!("Peripherals taken");

    // Configure the system clock
    // We'll use the default internal oscillator at 8MHz
    let mut rcc = dp.RCC.configure().freeze(&mut dp.FLASH);
    info!("Clocks configured");

    // Create delay instance
    let mut delay = Delay::new(cp.SYST, &rcc);

    // Initialize ADC
    let mut adc = Adc::new(dp.ADC, &mut rcc);
    info!("ADC initialized");

    // Configure LED pin
    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut user_led = cortex_m::interrupt::free(|cs| {
        gpioa.pa5.into_push_pull_output(cs).downgrade()
    });
    info!("LED pin configured (PA5: Push-Pull Output)");

    let mut blink_counter = 0;

    loop {
        // Read temperature using the internal temperature sensor
        // VTemp::read returns temperature in tenths of degrees Celsius
        let temp_tenths = VTemp::read(&mut adc, Some(&mut delay));
        let temp_celsius = temp_tenths as f32 / 10.0;

        user_led.toggle().unwrap();
        delay.delay_ms(500_u16);
        user_led.toggle().unwrap();
        delay.delay_ms(500_u16);
        blink_counter += 1;

        info!("LED blinked {} times, Temperature: {}°C", blink_counter, temp_celsius);
    }
}
