/// Configuration constants for the application

use embassy_sync::once_lock::OnceLock;
use crate::drivers::mcu_temp::TemperatureCalibration;

/// Initial LED blink interval in milliseconds
pub const INITIAL_BLINK_MS: u32 = 1000;

/// Minimum blink interval in milliseconds
pub const MIN_BLINK_MS: u32 = 50;

/// Main task heartbeat interval in milliseconds
pub const MAIN_HEARTBEAT_INTERVAL_MS: u64 = 10000;

/// How often to log blink counter (every N blinks)
pub const BLINK_LOG_FREQUENCY: u32 = 10;

/// Temperature reading interval in milliseconds
pub const MCU_TEMP_READING_INTERVAL_MS: u64 = 4000;

/// SHT31 sensor reading interval in milliseconds
pub const SHT31_READING_INTERVAL_MS: u64 = 3000;

/// VDD voltage during factory calibration (always 3.3V)
pub const MCU_VDDA_CALIB_MV: u32 = 3300;

/// MCU Temperature calibration points
pub const MCU_TEMP_CALIB_LOW: f32 = 30.0;
pub const MCU_TEMP_CALIB_HIGH: f32 = 110.0;

/// Factory calibration addresses from STM32F072 datasheet
const MCU_TEMP30_CAL_ADDR: *const u16 = 0x1FFF_F7B8 as *const u16;
const MCU_TEMP110_CAL_ADDR: *const u16 = 0x1FFF_F7C2 as *const u16;
const MCU_VREFINT_CAL_ADDR: *const u16 = 0x1FFF_F7BA as *const u16;

static FACTORY_CALIBRATION: OnceLock<TemperatureCalibration> = OnceLock::new();

/// Read mcu temperature factory calibration values from system memory
/// 
/// These values are programmed by ST during production and stored
/// in read-only system memory. They are used to compensate for
/// process variations and improve temperature measurement accuracy.
pub fn get_mcu_temp_factory_calibration() -> &'static TemperatureCalibration {
    FACTORY_CALIBRATION.get_or_init(|| {
        let (temp30_cal, temp110_cal, vrefint_cal) = unsafe {
            (
                core::ptr::read_volatile(MCU_TEMP30_CAL_ADDR),
                core::ptr::read_volatile(MCU_TEMP110_CAL_ADDR),
                core::ptr::read_volatile(MCU_VREFINT_CAL_ADDR),
            )
        };

        TemperatureCalibration {
            temp30_cal,
            temp110_cal,
            vrefint_cal,
        }
    })
}
