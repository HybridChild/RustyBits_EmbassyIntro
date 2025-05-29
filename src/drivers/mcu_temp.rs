use crate::config::{
    MCU_TEMP30_CAL_ADDR,
    MCU_TEMP110_CAL_ADDR,
    MCU_VREFINT_CAL_ADDR,
    MCU_VDDA_CALIB_MV,
    MCU_TEMP_CALIB_LOW,
    MCU_TEMP_CALIB_HIGH
};

/// Temperature sensor calibration data structure
#[derive(Debug, Clone, Copy)]
pub struct TemperatureCalibration {
    pub temp30_cal: u16,   // ADC value at 30°C
    pub temp110_cal: u16,  // ADC value at 110°C
    pub vrefint_cal: u16,  // VREFINT at 3.3V
}

/// Read factory calibration values from system memory
/// 
/// These values are programmed by ST during production and stored
/// in read-only system memory. They are used to compensate for
/// process variations and improve temperature measurement accuracy.
/// 
/// Returns: (temp30_cal, temp110_cal, vrefint_cal)
pub fn read_factory_calibration() -> (u16, u16, u16) {
    unsafe {
        let temp30_cal = core::ptr::read_volatile(MCU_TEMP30_CAL_ADDR);
        let temp110_cal = core::ptr::read_volatile(MCU_TEMP110_CAL_ADDR);
        let vrefint_cal = core::ptr::read_volatile(MCU_VREFINT_CAL_ADDR);

        (temp30_cal, temp110_cal, vrefint_cal)
    }
}

/// Read factory calibration values as a structured type
pub fn read_calibration_data() -> TemperatureCalibration {
    let (temp30_cal, temp110_cal, vrefint_cal) = read_factory_calibration();
    
    TemperatureCalibration {
        temp30_cal,
        temp110_cal,
        vrefint_cal,
    }
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
    let (temp30_cal, temp110_cal, vrefint_cal) = read_factory_calibration();

    // Calculate actual VDD voltage using factory VREFINT calibration
    let vdda_actual = (MCU_VDDA_CALIB_MV * vrefint_cal as u32) / vref_sample as u32;

    // Compensate temperature reading for actual VDD voltage
    let temp_compensated = (temp_sample as u32 * vdda_actual) / MCU_VDDA_CALIB_MV;

    // Calculate temperature using factory calibration and linear interpolation
    // Formula from STM32F072 reference manual:
    // Temperature = 30°C + ((ADC_DATA - TS_CAL1) * (110°C - 30°C)) / (TS_CAL2 - TS_CAL1)
    MCU_TEMP_CALIB_LOW + 
        ((temp_compensated as f32 - temp30_cal as f32) * (MCU_TEMP_CALIB_HIGH - MCU_TEMP_CALIB_LOW)) / 
        (temp110_cal as f32 - temp30_cal as f32)
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
