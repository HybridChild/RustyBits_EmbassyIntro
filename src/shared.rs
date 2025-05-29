use embassy_stm32::adc::Adc;
use embassy_stm32::peripherals::ADC1;
use embassy_sync::blocking_mutex::raw::{ThreadModeRawMutex, CriticalSectionRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;

/// Button events that can be signaled between tasks
#[derive(Clone, Copy)]
pub enum ButtonEvent {
    Pressed,
}

/// Global signal for button events
pub static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, ButtonEvent> = Signal::new();

/// Type alias for shared ADC wrapped in a mutex for safe concurrent access
pub type SharedAdc = Mutex<ThreadModeRawMutex, Option<Adc<'static, ADC1>>>;

/// Global static ADC instance
pub static SHARED_ADC: SharedAdc = Mutex::new(None);
