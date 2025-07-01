#![no_std]
#![no_main]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::adc::{Adc, SampleTime, Resolution};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{I2c, Config as I2cConfig};
use embassy_stm32::time::Hertz;
use embassy_stm32::mode::Blocking;
use embassy_stm32::{bind_interrupts, Config};
use embassy_stm32::peripherals::{ADC1, I2C1};
use embassy_time::Timer;

// Import our modules
use rusty_bits_embassy_intro::config::MAIN_HEARTBEAT_INTERVAL_MS;
use rusty_bits_embassy_intro::shared::SHARED_ADC;
use rusty_bits_embassy_intro::tasks::{button_task, blink_task, mcu_temp_task, sht31_task};
use rusty_bits_embassy_intro::config::get_mcu_temp_factory_calibration;

bind_interrupts!(struct Irqs {
    ADC1_COMP => embassy_stm32::adc::InterruptHandler<ADC1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start running main executor");

    // Initialize the microcontroller
    let p = embassy_stm32::init(Config::default());
    info!("Peripherals initialized");

    // Initialize ADC
    let adc = initialize_adc(p.ADC1);
    let mcu_temp_channel = adc.enable_temperature();
    let vref_channel = adc.enable_vref();

    // Store ADC in the global static
    *SHARED_ADC.lock().await = Some(adc);
    info!("ADC initialized and stored in global mutex");

    // Initialize I2C for SHT31
    let i2c = initialize_i2c(p.I2C1, p.PB8, p.PB9);
    info!("I2C initialized for SHT31");

    // Display factory calibration values at startup
    display_factory_calibration();

    // Initialize peripherals
    let led = Output::new(p.PA5, Level::Low, Speed::Low);
    info!("LED configured on PA5");

    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::None);
    info!("Button configured on PC13");

    // Spawn all tasks
    spawn_tasks(&spawner, button, led, mcu_temp_channel, vref_channel, i2c).unwrap();

    // Main task heartbeat
    run_main_loop().await;
}

/// Initialize ADC with proper settings for temperature sensor
fn initialize_adc(adc_peripheral: ADC1) -> Adc<'static, ADC1> {
    let mut adc = Adc::new(adc_peripheral, Irqs);

    // Set sampling time (must be >= 4µs for temperature sensor per datasheet)
    adc.set_sample_time(SampleTime::CYCLES239_5);
    adc.set_resolution(Resolution::BITS12);

    info!("ADC initialized with 239.5 cycle sampling time");
    adc
}

/// Initialize I2C with standard 100kHz frequency
fn initialize_i2c(
    i2c_peripheral: I2C1,
    scl_pin: embassy_stm32::peripherals::PB8,
    sda_pin: embassy_stm32::peripherals::PB9,
) -> I2c<'static, Blocking> {
    let i2c_config = I2cConfig::default();

    I2c::new_blocking(i2c_peripheral, scl_pin, sda_pin, Hertz(100_000), i2c_config)
}

/// Display factory calibration values at startup
fn display_factory_calibration() {
    let factory_calibraion = get_mcu_temp_factory_calibration();
    
    info!("Factory calibration values:");
    info!("  TEMP30_CAL: {} (ADC value at 30°C)", factory_calibraion.temp30_cal);
    info!("  TEMP110_CAL: {} (ADC value at 110°C)", factory_calibraion.temp110_cal);
    info!("  VREFINT_CAL: {} (VREFINT at 3.3V)", factory_calibraion.vrefint_cal);
}

/// Spawn all application tasks
fn spawn_tasks(
    spawner: &Spawner,
    button: ExtiInput<'static>,
    led: Output<'static>,
    temp_channel: embassy_stm32::adc::Temperature,
    vref_channel: embassy_stm32::adc::Vref,
    i2c: I2c<'static, Blocking>,
) -> Result<(), embassy_executor::SpawnError> {
    spawner.spawn(button_task(button))?;
    spawner.spawn(blink_task(led))?;
    spawner.spawn(mcu_temp_task(temp_channel, vref_channel))?;
    spawner.spawn(sht31_task(i2c))?;

    info!("All tasks spawned successfully");
    Ok(())
}

/// Main task loop - provides periodic heartbeat
async fn run_main_loop() -> ! {
    loop {
        Timer::after_millis(MAIN_HEARTBEAT_INTERVAL_MS).await;
        info!("Main task heartbeat - all tasks running independently");
    }
}
