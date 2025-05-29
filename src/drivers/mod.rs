pub mod mcu_temp;

// Re-export driver functions for easier access
pub use mcu_temp::{read_factory_calibration, calculate_temperature};
