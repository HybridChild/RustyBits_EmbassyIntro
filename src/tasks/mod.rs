pub mod button;
pub mod blink;
pub mod mcu_temp;

// Re-export task functions for easier access
pub use button::button_task;
pub use blink::blink_task;
pub use mcu_temp::mcu_temp_task;