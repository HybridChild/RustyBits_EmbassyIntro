use crate::config::{
    MCU_VDDA_CALIB_MV,
    MCU_TEMP_CALIB_LOW,
    MCU_TEMP_CALIB_HIGH,
    get_mcu_temp_factory_calibration,
};

/// Temperature sensor calibration data structure
#[derive(Debug, Clone, Copy)]
pub struct TemperatureCalibration {
    pub temp30_cal: u16,   // ADC value at 30°C
    pub temp110_cal: u16,  // ADC value at 110°C
    pub vrefint_cal: u16,  // VREFINT at 3.3V
}

/// Calculate temperature from ADC readings using factory calibration
/// 
/// This function implements the temperature calculation formula from the
/// STM32F072 reference manual, including compensation for actual VDD voltage.
/// 
/// # Arguments
/// * `temp_sample` - Raw ADC reading from temperature sensor
/// * `vref_sample` - Raw ADC reading from internal voltage reference
/// 
/// # Returns
/// Temperature in degrees Celsius as a floating-point value
pub fn calculate_temperature(temp_sample: u16, vref_sample: u16) -> f32 {
    // Read factory calibration values from flash
    let factory_calibraion = get_mcu_temp_factory_calibration();

    calculate_temperature_with_cal(temp_sample, vref_sample, factory_calibraion)
}

/// Calculate temperature with structured calibration data
pub fn calculate_temperature_with_cal(
    temp_sample: u16, 
    vref_sample: u16, 
    cal: &TemperatureCalibration
) -> f32 {
    // Calculate actual VDD voltage using factory VREFINT calibration
    let vdda_actual = (MCU_VDDA_CALIB_MV * cal.vrefint_cal as u32) / vref_sample as u32;

    // Compensate temperature reading for actual VDD voltage
    let temp_compensated = (temp_sample as u32 * vdda_actual) / MCU_VDDA_CALIB_MV;

    // Calculate temperature using factory calibration and linear interpolation
    MCU_TEMP_CALIB_LOW + 
        ((temp_compensated as f32 - cal.temp30_cal as f32) * (MCU_TEMP_CALIB_HIGH - MCU_TEMP_CALIB_LOW)) / 
        (cal.temp110_cal as f32 - cal.temp30_cal as f32)
}
