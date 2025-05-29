#![no_std]

pub mod config;
pub mod shared;
pub mod tasks;
pub mod drivers;

// Re-export commonly used items
pub use shared::{ButtonEvent, BUTTON_SIGNAL, SHARED_ADC, SharedAdc};
pub use config::*;
