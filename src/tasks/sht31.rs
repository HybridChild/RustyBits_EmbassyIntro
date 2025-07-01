// src/tasks/sht31.rs
use defmt::info;
use embassy_executor::task;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_time::Timer;
use crate::config::SHT31_READING_INTERVAL_MS;

const SHT31_ADDR: u8 = 0x44; // Address when ADDR pin is low
const CMD_MEASURE_MEDIUM: [u8; 2] = [0x24, 0x0B]; // Single shot, medium repeatability, no clock stretch
const MEASURE_DELAY_MS: u64 = 6; // Medium repeatability max duration

/// Task that periodically reads temperature and humidity from SHT31 sensor
/// 
/// This implementation directly uses I2C commands instead of the sht3x crate
/// to avoid blocking delays and work properly with Embassy's async model.
#[task]
pub async fn sht31_task(mut i2c: I2c<'static, Blocking>) {
    info!("SHT31 task started");
    
    let mut reading_counter = 0;

    // Try to read status register to verify connection
    match read_status(&mut i2c) {
        Ok(status) => info!("SHT31 sensor connected, status: 0x{:04X}", status),
        Err(_) => info!("Warning: Could not read SHT31 status register"),
    }

    loop {
        Timer::after_millis(SHT31_READING_INTERVAL_MS).await;

        match measure_temp_humidity(&mut i2c) {
            Ok((temperature, humidity)) => {
                reading_counter += 1;

                info!(
                    "SHT31 Reading #{}: Temperature: {}.{}°C, Humidity: {}.{}%",
                    reading_counter,
                    (temperature as i32),
                    ((temperature * 10.0) as i32) % 10,
                    (humidity as i32),
                    ((humidity * 10.0) as i32) % 10
                );
            }
            Err(e) => {
                info!("SHT31 measurement error: {:?}", e);
            }
        }
    }
}

/// Read status register from SHT31
fn read_status(i2c: &mut I2c<'_, Blocking>) -> Result<u16, embassy_stm32::i2c::Error> {
    let cmd = [0xF3, 0x2D]; // Status command
    i2c.blocking_write(SHT31_ADDR, &cmd)?;
    
    let mut buf = [0u8; 3]; // 2 bytes data + 1 byte CRC
    i2c.blocking_read(SHT31_ADDR, &mut buf)?;
    
    // Convert big-endian bytes to u16
    Ok(u16::from_be_bytes([buf[0], buf[1]]))
}

/// Perform temperature and humidity measurement
fn measure_temp_humidity(
    i2c: &mut I2c<'_, Blocking>
) -> Result<(f32, f32), embassy_stm32::i2c::Error> {
    // Send measurement command
    i2c.blocking_write(SHT31_ADDR, &CMD_MEASURE_MEDIUM)?;
    
    // Wait for measurement to complete (blocking delay is OK here since it's short)
    cortex_m::asm::delay(MEASURE_DELAY_MS as u32 * 8000); // ~6ms delay
    
    // Read result (6 bytes: temp_high, temp_low, temp_crc, hum_high, hum_low, hum_crc)
    let mut buf = [0u8; 6];
    i2c.blocking_read(SHT31_ADDR, &mut buf)?;
    
    // Extract temperature and humidity values using built-in byte operations
    let temp_raw = u16::from_be_bytes([buf[0], buf[1]]);
    let hum_raw = u16::from_be_bytes([buf[3], buf[4]]);
    
    // Convert using SHT3x formulas
    let temperature = -45.0 + 175.0 * (temp_raw as f32) / 65535.0;
    let humidity = 100.0 * (hum_raw as f32) / 65535.0;
    
    Ok((temperature, humidity))
}
