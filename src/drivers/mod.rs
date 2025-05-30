pub mod mcu_temp;
pub mod sht31;

// Re-export driver functions for easier access
pub use mcu_temp::{calculate_temperature};
