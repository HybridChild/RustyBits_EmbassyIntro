use defmt::info;
use embassy_executor::task;
use embassy_time::Timer;
use crate::config::MCU_TEMP_READING_INTERVAL_MS;
use crate::shared::SHARED_ADC;
use crate::drivers::mcu_temp::calculate_temperature;

/// Task that periodically reads the internal temperature sensor
/// 
/// Reads both the temperature sensor and voltage reference channels,
/// then calculates the actual temperature using factory calibration data.
#[task]
pub async fn mcu_temp_task(
    mut temp_channel: embassy_stm32::adc::Temperature,
    mut vref_channel: embassy_stm32::adc::Vref,
) {
    info!("Temperature task started");

    loop {
        // Temperature readings don't need to be as frequent
        Timer::after_millis(MCU_TEMP_READING_INTERVAL_MS).await;

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

        // Convert to tenths for integer display
        let temp_tenths = (temp_celsius * 10.0) as i32;

        info!(
            "MCU Temperature: {}.{}°C",
            temp_tenths / 10,
            temp_tenths % 10,
        );
    }
}
