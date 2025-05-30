use embedded_sht3x::{Sht3x, Repeatability, DEFAULT_I2C_ADDRESS};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_time::Delay;
use defmt::info;

/// SHT31 sensor wrapper
pub struct Sht31Sensor {
    sensor: Sht3x<I2c<'static, Blocking>, Delay>,
}

impl Sht31Sensor {
    /// Create a new SHT31 sensor instance
    pub fn new(i2c: I2c<'static, Blocking>) -> Self {
        let mut sensor = Sht3x::new(i2c, DEFAULT_I2C_ADDRESS, Delay);
        sensor.repeatability = Repeatability::High; // Best accuracy

        Self { sensor }
    }

    /// Initialize the sensor (reset and check status)
    pub fn init(&mut self) -> Result<(), embedded_sht3x::Error<embassy_stm32::i2c::Error>> {
        // Reset the sensor to ensure clean state
        self.sensor.reset()?;

        // Clear any pending status
        self.sensor.clear_status()?;

        // Check sensor status
        let _status = self.sensor.get_status()?;
        info!("SHT31 initialized successfully");

        Ok(())
    }

    /// Read temperature and humidity
    pub fn read_measurement(&mut self) -> Result<(f32, f32), embedded_sht3x::Error<embassy_stm32::i2c::Error>> {
        let measurement = self.sensor.single_measurement()?;
        Ok((measurement.temperature, measurement.humidity))
    }

    /// Enable internal heater (useful for diagnostics)
    pub fn enable_heater(&mut self) -> Result<(), embedded_sht3x::Error<embassy_stm32::i2c::Error>> {
        self.sensor.enable_heater()
    }

    /// Disable internal heater
    pub fn disable_heater(&mut self) -> Result<(), embedded_sht3x::Error<embassy_stm32::i2c::Error>> {
        self.sensor.disable_heater()
    }
}
