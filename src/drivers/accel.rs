//! QMI8658 accelerometer.

use super::Error;
use crate::hal::i2c::I2c;
use crate::hal::timer::Delay;

const WHO_AM_I: u8 = 0x00;
const CTRL1: u8 = 0x02;
const CTRL2: u8 = 0x03;
const CTRL5: u8 = 0x06;
const CTRL7: u8 = 0x08;
const STATUS0: u8 = 0x2e;
const ACCEL_X_LOW: u8 = 0x35;
const AUTO_INCREMENT: u8 = 0x40;
const SENSOR_DISABLE: u8 = 0x01;
const ACCEL_ENABLE: u8 = 0x01;
const ACCEL_2G_500HZ: u8 = 0x04;
const ACCEL_READY: u8 = 0x01;
const DEVICE_ID: u8 = 0x05;

/// A discovered accelerometer, in standby between readings
pub struct Accel {
    address: u8,
}

impl Accel {
    /// Find the chip at either of its addresses and put it in standby
    pub fn discover(bus: &mut impl I2c) -> Result<Self, Error> {
        for address in [0x6a, 0x6b] {
            let accel = Accel { address };
            if accel.read_register(bus, WHO_AM_I) == Ok(DEVICE_ID) {
                return accel.standby(bus).map(|()| accel);
            }
        }
        Err(Error::Bus)
    }

    /// Wake, average four samples, then verify standby
    pub fn read(&self, bus: &mut impl I2c, delay: &mut impl Delay) -> Result<[i16; 3], Error> {
        let sums = self.collect_samples(bus, delay);
        let asleep = self.standby(bus);
        let sums = sums?;
        asleep?;
        Ok(sums.map(|sum| (sum / 4) as i16))
    }

    fn read_register(&self, bus: &mut impl I2c, register: u8) -> Result<u8, Error> {
        let mut value = 0;
        bus.read(self.address, &[register], core::slice::from_mut(&mut value))?;
        Ok(value)
    }

    fn write_register(&self, bus: &mut impl I2c, register: u8, value: u8) -> Result<(), Error> {
        Ok(bus.write(self.address, &[register, value])?)
    }

    fn standby(&self, bus: &mut impl I2c) -> Result<(), Error> {
        let mut outcome = Err(Error::Bus);
        for _ in 0..3 {
            // Attempt both writes even when one fails, and retry transient NACKs
            let disabled = self.write_register(bus, CTRL7, 0);
            let stopped = self.write_register(bus, CTRL1, AUTO_INCREMENT | SENSOR_DISABLE);
            // Read back before claiming shutdown succeeded
            outcome = disabled.and(stopped).and_then(|()| {
                let ctrl7 = self.read_register(bus, CTRL7)?;
                let ctrl1 = self.read_register(bus, CTRL1)?;
                if ctrl7 == 0 && ctrl1 == AUTO_INCREMENT | SENSOR_DISABLE {
                    Ok(())
                } else {
                    Err(Error::Unverified)
                }
            });
            if outcome.is_ok() {
                break;
            }
        }
        outcome
    }

    fn collect_samples(
        &self,
        bus: &mut impl I2c,
        delay: &mut impl Delay,
    ) -> Result<[i32; 3], Error> {
        // Auto-increment, little endian, +/-2 g at 500 Hz; gyro remains off
        self.write_register(bus, CTRL1, AUTO_INCREMENT)?;
        self.write_register(bus, CTRL2, ACCEL_2G_500HZ)?;
        self.write_register(bus, CTRL5, 0)?;
        self.write_register(bus, CTRL7, ACCEL_ENABLE)?;
        delay.delay_us(20_000);
        let mut sums = [0; 3];
        for _ in 0..4 {
            delay.delay_us(3000);
            if self.read_register(bus, STATUS0)? & ACCEL_READY == 0 {
                return Err(Error::Invalid);
            }
            let mut data = [0; 6];
            bus.read(self.address, &[ACCEL_X_LOW], &mut data)?;
            for (sum, axis) in sums.iter_mut().zip(data.as_chunks().0) {
                *sum += i16::from_le_bytes(*axis) as i32;
            }
        }
        Ok(sums)
    }
}
