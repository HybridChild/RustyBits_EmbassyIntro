#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime, Resolution};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::{bind_interrupts, Config};
use embassy_stm32::peripherals::ADC1;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_COMP => embassy_stm32::adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting Embassy temperature sensor example");

    // Initialize the microcontroller
    let p = embassy_stm32::init(Config::default());
    
    info!("Peripherals initialized");

    // Initialize ADC
    let mut adc = Adc::new(p.ADC1, Irqs);
    
    // Set sampling time (must be >= 4µs for temperature sensor per datasheet)
    adc.set_sample_time(SampleTime::CYCLES239_5);
    adc.set_resolution(Resolution::BITS12);
    
    info!("ADC initialized with 239.5 cycle sampling time");

    // Enable temperature sensor and voltage reference
    let mut temp_channel = adc.enable_temperature();
    let mut vref_channel = adc.enable_vref();
    
    // Initialize LED on PA5
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);
    info!("LED configured on PA5");
    let mut blink_counter = 0;

    // Read and display factory calibration values once at startup
    let (temp30_cal, temp110_cal, vrefint_cal) = read_factory_calibration();
    info!("Factory calibration values:");
    info!("  TEMP30_CAL: {} (ADC value at 30°C)", temp30_cal);
    info!("  TEMP110_CAL: {} (ADC value at 110°C)", temp110_cal);
    info!("  VREFINT_CAL: {} (VREFINT at 3.3V)", vrefint_cal);

    loop {
        // Read voltage reference to calculate actual VDD
        let vref_sample = adc.read(&mut vref_channel).await;
        
        // Read temperature sensor
        let temp_sample = adc.read(&mut temp_channel).await;
        
        // Calculate temperature using the calibration formula
        // Using factory calibration values from the STM32F072 datasheet
        let temp_celsius = calculate_temperature(temp_sample, vref_sample);
        
        // Blink LED
        led.set_high();
        Timer::after_millis(400).await;
        led.set_low();
        Timer::after_millis(400).await;
        
        blink_counter += 1;

        // Convert to tenths for integer display
        let temp_tenths = (temp_celsius * 10.0) as i32;
        
        info!(
            "Blink: {}, Temperature: {}.{}°C", 
            blink_counter, 
            temp_tenths / 10,
            temp_tenths % 10
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
