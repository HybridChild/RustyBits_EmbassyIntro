use defmt::info;
use embassy_executor::task;
use embassy_stm32::exti::ExtiInput;
use crate::shared::{ButtonEvent, BUTTON_SIGNAL};

/// Task that handles button press events
/// 
/// Waits for falling edge on the button pin and signals other tasks
/// when a button press is detected.
#[task]
pub async fn button_task(mut button: ExtiInput<'static>) {
    info!("Button task started");

    loop {
        // Check if button got pressed
        button.wait_for_falling_edge().await;
        BUTTON_SIGNAL.signal(ButtonEvent::Pressed);
        info!("Button pressed...");
    }
}
