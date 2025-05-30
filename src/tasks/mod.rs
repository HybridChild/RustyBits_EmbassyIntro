pub mod button;
pub mod blink;
pub mod mcu_temp;
pub mod sht31;

// Re-export task functions for easier access
pub use button::button_task;
pub use blink::blink_task;
pub use mcu_temp::mcu_temp_task;
pub use sht31::sht31_task;
