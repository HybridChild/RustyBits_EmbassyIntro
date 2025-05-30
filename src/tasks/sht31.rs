use defmt::info;
use embassy_executor::task;
use embassy_time::Timer;
use crate::config::SHT31_READING_INTERVAL_MS;
use crate::drivers::sht31::Sht31Sensor;

/// Task that periodically reads the SHT31 temperature and humidity sensor
#[task]
pub async fn sht31_task(mut sensor: Sht31Sensor) {
    info!("SHT31 task started");

    // Initialize the sensor
    match sensor.init() {
        Ok(_) => info!("SHT31 sensor initialized successfully"),
        Err(e) => {
            match e {
                embedded_sht3x::Error::I2c(_) => info!("Failed to initialize SHT31 sensor: I2C error"),
                embedded_sht3x::Error::BadCrc => info!("Failed to initialize SHT31 sensor: Bad CRC"),
            }
            return;
        }
    }

    loop {
        Timer::after_millis(SHT31_READING_INTERVAL_MS).await;

        match sensor.read_measurement() {
            Ok((temperature, humidity)) => {
                // Convert to tenths for integer display
                let temp_tenths = (temperature * 10.0) as i32;
                let humidity_tenths = (humidity * 10.0) as i32;

                info!(
                    "Room Temperature: {}.{}°C, Humidity: {}.{}%",
                    temp_tenths / 10,
                    temp_tenths % 10,
                    humidity_tenths / 10,
                    humidity_tenths % 10,
                );
            }
            Err(e) => {
                match e {
                    embedded_sht3x::Error::I2c(_) => info!("Failed to read SHT31 sensor: I2C error"),
                    embedded_sht3x::Error::BadCrc => info!("Failed to read SHT31 sensor: Bad CRC"),
                }
            }
        }
    }
}
