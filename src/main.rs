#![no_std]
#![no_main]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::adc::{Adc, SampleTime, Resolution};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::{bind_interrupts, Config};
use embassy_stm32::peripherals::ADC1;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, WithTimeout};

bind_interrupts!(struct Irqs {
    ADC1_COMP => embassy_stm32::adc::InterruptHandler<ADC1>;
});

enum ButtonEvent {
    Pressed,
}

static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, ButtonEvent> = Signal::new();

// Shared ADC wrapped in a mutex for safe concurrent access
type SharedAdc = Mutex<ThreadModeRawMutex, Option<Adc<'static, ADC1>>>;
// Global static ADC instance
static SHARED_ADC: SharedAdc = Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start running main executor");

    // Initialize the microcontroller
    let p = embassy_stm32::init(Config::default());
    info!("Peripherals initialized");

    // Initialize ADC
    let mut adc = Adc::new(p.ADC1, Irqs);
    // Set sampling time (must be >= 4µs for temperature sensor per datasheet)
    adc.set_sample_time(SampleTime::CYCLES239_5);
    adc.set_resolution(Resolution::BITS12);
    // Enable temperature sensor and voltage reference
    let temp_channel = adc.enable_temperature();
    let vref_channel = adc.enable_vref();
    // Store ADC in the global static
    *SHARED_ADC.lock().await = Some(adc);
    info!("ADC initialized with 239.5 cycle sampling time");
    // Read and display factory calibration values once at startup
    let (temp30_cal, temp110_cal, vrefint_cal) = read_factory_calibration();
    info!("Read factory calibration values:");
    info!("  TEMP30_CAL: {} (ADC value at 30°C)", temp30_cal);
    info!("  TEMP110_CAL: {} (ADC value at 110°C)", temp110_cal);
    info!("  VREFINT_CAL: {} (VREFINT at 3.3V)", vrefint_cal);

    // Initialize LED on PA5
    let led = Output::new(p.PA5, Level::Low, Speed::Low);
    info!("LED configured on PA5");

    // Configure the button pin and obtain handler.
    // On the NUCLEO-F072RB there is a button connected to pin PC13.
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::None);
    info!("Button configured on PC13");

    // Spawn tasks
    spawner.spawn(button_task(button)).unwrap();
    spawner.spawn(blink_task(led)).unwrap();
    spawner.spawn(temperature_task(temp_channel, vref_channel)).unwrap();

    // Main task can do other work or just wait
    loop {
        Timer::after_millis(10000).await;
        info!("Main task heartbeat - all tasks running independently");
    }
}

#[embassy_executor::task]
async fn button_task(mut button: ExtiInput<'static>) {

    loop {
        // Check if button got pressed
        button.wait_for_falling_edge().await;
        BUTTON_SIGNAL.signal(ButtonEvent::Pressed);
        info!("Button pressed...");
    }
}

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    const INITIAL_BLINK_MS: u32 = 1000;
    let mut blink_ms = INITIAL_BLINK_MS;
    let mut blink_counter = 0;
    let mut next_level = Level::High;

    loop {
        let delay = Duration::from_millis(blink_ms as u64);

        if let Ok(_event) = BUTTON_SIGNAL.wait().with_timeout(delay).await {
            next_level = Level::High;
            blink_ms >>= 1;
            // If updated delay value drops below 50ms then reset it back to starting value
            if blink_ms < 50 {
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
            if blink_counter % 10 == 0 {
                info!("LED blinked {} times", blink_counter);
            }
        } else {
            next_level = Level::High;
        }
    }
}

#[embassy_executor::task]
async fn temperature_task(
    mut temp_channel: embassy_stm32::adc::Temperature,
    mut vref_channel: embassy_stm32::adc::Vref,
) {
    let mut reading_counter = 0;

    loop {
        // Temperature readings don't need to be as frequent
        Timer::after_millis(4000).await;

        let (vref_sample, temp_sample) = {
            // Lock the global ADC mutex for the duration of both readings
            let mut adc_guard = SHARED_ADC.lock().await;
            let adc = adc_guard.as_mut().unwrap();

            // Read voltage reference to calculate actual VDD
            let vref_sample = adc.read(&mut vref_channel).await;

            // Read temperature sensor
            let temp_sample = adc.read(&mut temp_channel).await;

            (vref_sample, temp_sample)
        }; // ADC mutex is automatically released here

        // Calculate temperature using the calibration formula
        let temp_celsius = calculate_temperature(temp_sample, vref_sample);

        reading_counter += 1;

        // Convert to tenths for integer display
        let temp_tenths = (temp_celsius * 10.0) as i32;

        info!(
            "Reading #{}: Temperature: {}.{}°C", 
            reading_counter,
            temp_tenths / 10,
            temp_tenths % 10,
        );
    }
}

fn read_factory_calibration() -> (u16, u16, u16) {
    // Factory calibration addresses from STM32F072 datasheet
    const TEMP30_CAL_ADDR: *const u16 = 0x1FFF_F7B8 as *const u16;
    const TEMP110_CAL_ADDR: *const u16 = 0x1FFF_F7C2 as *const u16;
    const VREFINT_CAL_ADDR: *const u16 = 0x1FFF_F7BA as *const u16;

    unsafe {
        let temp30_cal = core::ptr::read_volatile(TEMP30_CAL_ADDR);
        let temp110_cal = core::ptr::read_volatile(TEMP110_CAL_ADDR);
        let vrefint_cal = core::ptr::read_volatile(VREFINT_CAL_ADDR);

        (temp30_cal, temp110_cal, vrefint_cal)
    }
}

fn calculate_temperature(temp_sample: u16, vref_sample: u16) -> f32 {
    // Read factory calibration values from flash
    let (temp30_cal, temp110_cal, vrefint_cal) = read_factory_calibration();

    // VDD voltage during factory calibration (always 3.3V)
    const VDDA_CALIB_MV: u32 = 3300;

    // Calculate actual VDD voltage using factory VREFINT calibration
    let vdda_actual = (VDDA_CALIB_MV * vrefint_cal as u32) / vref_sample as u32;

    // Compensate temperature reading for actual VDD voltage
    let temp_compensated = (temp_sample as u32 * vdda_actual) / VDDA_CALIB_MV;

    // Calculate temperature using factory calibration and linear interpolation
    // Formula from STM32F072 reference manual
    30.0 + ((temp_compensated as f32 - temp30_cal as f32) * (110.0 - 30.0)) / (temp110_cal as f32 - temp30_cal as f32)
}
