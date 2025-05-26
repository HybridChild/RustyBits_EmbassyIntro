#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f0xx_hal::{
    delay::Delay,
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    // Initialize RTT
    rtt_init_print!();
    rprintln!("Starting program...");

    // Get access to cortex-m preipherals
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Get access to the device peripherals
    let mut dp = pac::Peripherals::take().unwrap();
    rprintln!("Peripherals taken");

    // Configure the system clock
    // We'll use the default internal oscillator at 8MHz
    let mut rcc = dp.RCC.configure().freeze(&mut dp.FLASH);
    rprintln!("Clocks configured");

    // Create delay instance
    let mut delay = Delay::new(cp.SYST, &rcc);

    // Configure LED pin
    let gpioa = dp.GPIOA.split(&mut rcc);
    let mut user_led = cortex_m::interrupt::free(|cs| {
        gpioa.pa5.into_push_pull_output(cs).downgrade()
    });
    rprintln!("LED pin configured (PA5: Push-Pull Output");

    let mut blink_counter = 0;

    loop {
        user_led.toggle().unwrap();
        delay.delay_ms(500_u16);
        user_led.toggle().unwrap();
        delay.delay_ms(500_u16);
        blink_counter += 1;
        rprintln!("LED blinked {} times", blink_counter);
    }
}
