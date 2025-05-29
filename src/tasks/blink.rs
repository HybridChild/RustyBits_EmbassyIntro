use defmt::info;
use embassy_executor::task;
use embassy_stm32::gpio::{Level, Output};
use embassy_time::{Duration, WithTimeout};
use crate::config::{INITIAL_BLINK_MS, MIN_BLINK_MS, BLINK_LOG_FREQUENCY};
use crate::shared::BUTTON_SIGNAL;

/// Task that blinks an LED with variable frequency
/// 
/// The blink rate changes when button press events are received.
/// Each button press halves the blink interval until it reaches
/// the minimum, then resets to the initial value.
#[task]
pub async fn blink_task(mut led: Output<'static>) {
    info!("Blink task started");

    let mut blink_ms = INITIAL_BLINK_MS;
    let mut blink_counter = 0;
    let mut next_level = Level::High;

    loop {
        let delay = Duration::from_millis(blink_ms as u64);

        // Wait for timeout or button press
        if let Ok(_event) = BUTTON_SIGNAL.wait().with_timeout(delay).await {
            next_level = Level::High;
            blink_ms >>= 1; // Halve the blink interval

            // If updated delay value drops below minimum, reset to initial value
            if blink_ms < MIN_BLINK_MS {
                blink_ms = INITIAL_BLINK_MS;
            }
            info!("Blink speed set to {}ms", blink_ms);
        }

        // Blink LED
        led.set_level(next_level);
        
        if next_level == Level::High {
            next_level = Level::Low;
            blink_counter += 1;

            // Log blink status less frequently to avoid spam
            if blink_counter % BLINK_LOG_FREQUENCY == 0 {
                info!("LED blinked {} times", blink_counter);
            }
        } else {
            next_level = Level::High;
        }
    }
}
