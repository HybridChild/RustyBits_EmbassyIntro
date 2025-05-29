/// Configuration constants for the application

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

/// Factory calibration addresses from STM32F072 datasheet
pub const MCU_TEMP30_CAL_ADDR: *const u16 = 0x1FFF_F7B8 as *const u16;
pub const MCU_TEMP110_CAL_ADDR: *const u16 = 0x1FFF_F7C2 as *const u16;
pub const MCU_VREFINT_CAL_ADDR: *const u16 = 0x1FFF_F7BA as *const u16;

/// VDD voltage during factory calibration (always 3.3V)
pub const MCU_VDDA_CALIB_MV: u32 = 3300;

/// MCU Temperature calibration points
pub const MCU_TEMP_CALIB_LOW: f32 = 30.0;
pub const MCU_TEMP_CALIB_HIGH: f32 = 110.0;
