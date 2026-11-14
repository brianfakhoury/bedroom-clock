use crate::drivers::Error;
use crate::drivers::accel::Accel;
use crate::hal::i2c::{I2c, NoResponse};
use crate::tests::fakes::Elapsed;

const SAMPLES: [[i16; 3]; 4] =
    [[-32768, 100, -1], [32767, 200, -2], [-100, 300, -3], [97, 400, -4]];

struct Chip {
    address: u8,
    registers: [u8; 256],
    samples_read: usize,
    shutdown_writes: u32,
    fail_next_shutdowns: u32,
    fail_reads: bool,
    fail_writes: bool,
    fail_sample: bool,
    fail_shutdown: bool,
    ignore_shutdown: bool,
}

impl Chip {
    fn at(address: u8) -> Self {
        Self {
            address,
            registers: [0; 256],
            samples_read: 0,
            shutdown_writes: 0,
            fail_next_shutdowns: 0,
            fail_reads: false,
            fail_writes: false,
            fail_sample: false,
            fail_shutdown: false,
            ignore_shutdown: false,
        }
    }

    fn is_shut_down(&self) -> bool {
        self.registers[8] == 0 && self.registers[2] == 0x41
    }

    fn read_xyz(&mut self, accel: &Accel) -> Result<[i16; 3], Error> {
        accel.read(self, &mut &Elapsed::default())
    }
}

impl I2c for Chip {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), NoResponse> {
        let &[register, value] = bytes else { panic!("expected one register value") };
        assert_eq!(address, self.address);
        if self.fail_writes {
            return Err(NoResponse);
        }
        let shutdown = (register == 8 && value == 0) || (register == 2 && value == 0x41);
        if shutdown {
            self.shutdown_writes += 1;
            if self.fail_shutdown {
                return Err(NoResponse);
            }
            if self.fail_next_shutdowns > 0 {
                self.fail_next_shutdowns -= 1;
                return Err(NoResponse);
            }
            if self.ignore_shutdown {
                return Ok(());
            }
        }
        self.registers[register as usize] = value;
        Ok(())
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        if self.fail_reads || address != self.address {
            return Err(NoResponse);
        }
        match (register, data) {
            ([0x00], [id]) => *id = 5,
            ([0x2e], [status]) if !self.fail_sample => *status = 1,
            ([0x2e], [_]) => return Err(NoResponse),
            ([register @ (2 | 8)], [value]) => *value = self.registers[*register as usize],
            ([0x35], data) => {
                let sample = SAMPLES[self.samples_read % 4];
                self.samples_read += 1;
                data.copy_from_slice(&sample.map(i16::to_le_bytes).concat());
            }
            _ => panic!("unexpected read of {register:?}"),
        }
        Ok(())
    }
}

fn discovered() -> (Accel, Chip) {
    let mut chip = Chip::at(0x6a);
    (Accel::discover(&mut chip).unwrap(), chip)
}

#[test]
fn discovery() {
    for address in [0x6a, 0x6b] {
        let mut chip = Chip::at(address);
        assert!(Accel::discover(&mut chip).is_ok());
        assert!(chip.is_shut_down());
        chip.fail_reads = true;
        assert!(Accel::discover(&mut chip).is_err());
    }
    let mut chip = Chip::at(0x6a);
    chip.fail_writes = true;
    assert!(Accel::discover(&mut chip).is_err());
    assert!(Accel::discover(&mut Chip::at(0)).is_err());
}

#[test]
fn every_sample_attempt_shuts_down() {
    let (accel, mut chip) = discovered();
    for attempt in 0..100 {
        chip.fail_sample = attempt % 2 == 1;
        let before = chip.shutdown_writes;
        assert_eq!(chip.read_xyz(&accel).is_ok(), !chip.fail_sample);
        assert_eq!(chip.shutdown_writes, before + 2);
        assert!(chip.is_shut_down());
    }
}

#[test]
fn sample_average() {
    let (accel, mut chip) = discovered();
    assert_eq!(chip.read_xyz(&accel), Ok([-1, 250, -2]));
    assert_eq!(chip.samples_read, 4);
    chip.fail_sample = true;
    assert_eq!(chip.read_xyz(&accel), Err(Error::Bus));
}

#[test]
fn shutdown_retries() {
    let (accel, mut chip) = discovered();
    chip.fail_next_shutdowns = 2;
    assert!(chip.read_xyz(&accel).is_ok());
    assert!(chip.is_shut_down());
}

#[test]
fn shutdown_requires_readback() {
    let (accel, mut chip) = discovered();
    chip.ignore_shutdown = true;
    // ACK without a state change must not pass
    assert_eq!(chip.read_xyz(&accel), Err(Error::Unverified));
}

#[test]
fn permanent_failure_is_bounded_and_recoverable() {
    let (accel, mut chip) = discovered();
    chip.fail_shutdown = true;
    let before = chip.shutdown_writes;
    assert!(chip.read_xyz(&accel).is_err());
    assert_eq!(chip.shutdown_writes, before + 6);
    // Reporting a bus failure cannot guarantee that hardware is off
    assert_eq!((chip.registers[8], chip.registers[2]), (1, 0x40));
    chip.fail_shutdown = false;
    assert!(chip.read_xyz(&accel).is_ok());
    assert!(chip.is_shut_down());
}
