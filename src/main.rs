#![no_std]
#![no_main]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{bind_interrupts, Config};
use embassy_stm32::adc::{Adc, SampleTime, Resolution};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{I2c, Config as I2cConfig};
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::peripherals::I2C1;
use embassy_time::Timer;
use embassy_stm32::time::Hertz;

// Import our modules
use rusty_bits_embassy_intro::config::MAIN_HEARTBEAT_INTERVAL_MS;
use rusty_bits_embassy_intro::drivers::sht31::Sht31Sensor;
use rusty_bits_embassy_intro::shared::SHARED_ADC;
use rusty_bits_embassy_intro::tasks::{button_task, blink_task, mcu_temp_task, sht31_task};
use rusty_bits_embassy_intro::config::get_mcu_temp_factory_calibration;

bind_interrupts!(struct Irqs {
    ADC1_COMP => embassy_stm32::adc::InterruptHandler<ADC1>;
    I2C1 => embassy_stm32::i2c::EventInterruptHandler<I2C1>, embassy_stm32::i2c::ErrorInterruptHandler<I2C1>;
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

    // Display factory calibration values at startup
    display_factory_calibration();

    // Initialize I2C1 for SHT31
    let i2c = initialize_i2c_blocking(p.I2C1, p.PB8, p.PB9);

    // Create SHT31 sensor
    let sht31_sensor = Sht31Sensor::new(i2c);

    // Initialize peripherals
    let led = Output::new(p.PA5, Level::Low, Speed::Low);
    info!("LED configured on PA5");

    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::None);
    info!("Button configured on PC13");

    // Spawn all tasks
    spawn_tasks(
        &spawner,
        button,
        led,
        mcu_temp_channel,
        vref_channel,
        sht31_sensor
    ).unwrap();

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

// Function to initialize I2C1 (blocking mode - no DMA):
fn initialize_i2c_blocking(
    i2c1: I2C1, 
    scl: embassy_stm32::peripherals::PB8, 
    sda: embassy_stm32::peripherals::PB9
) -> I2c<'static, embassy_stm32::mode::Blocking> {
    let mut config = I2cConfig::default();
    config.sda_pullup = true;
    config.scl_pullup = true;
    
    let i2c = I2c::new_blocking(
        i2c1,
        scl,    // PB8 as SCL
        sda,    // PB9 as SDA
        Hertz(100_000), // 100 kHz standard speed
        config,
    );
    
    info!("I2C1 initialized (blocking) on PB8(SCL)/PB9(SDA) at 100kHz");
    i2c
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
    sht31_sensor: Sht31Sensor,
) -> Result<(), embassy_executor::SpawnError> {
    spawner.spawn(button_task(button))?;
    spawner.spawn(blink_task(led))?;
    spawner.spawn(mcu_temp_task(temp_channel, vref_channel))?;
    spawner.spawn(sht31_task(sht31_sensor))?;

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
