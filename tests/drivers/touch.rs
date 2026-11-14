use crate::drivers::{Error, touch};
use crate::hal::i2c::{I2c, NoResponse};

/// Answers every report request with fixed bytes, if present
struct Controller(Option<[u8; 10]>);

impl I2c for Controller {
    fn write(&mut self, _: u8, _: &[u8]) -> Result<(), NoResponse> {
        panic!("reading touch reports needs no writes")
    }

    fn read(&mut self, address: u8, register: &[u8], data: &mut [u8]) -> Result<(), NoResponse> {
        assert_eq!((address, register), (0x5a, &[0xd0, 0x00][..]));
        data.copy_from_slice(&self.0.ok_or(NoResponse)?);
        Ok(())
    }
}

fn report(kind: u8, fingers: u8, signature: u8) -> Controller {
    Controller(Some([kind, 0, 0, 0, 0, fingers, signature, 0, 0, 0]))
}

#[test]
fn contact_needs_a_finger_in_a_valid_report() {
    assert_eq!(touch::read(&mut report(0x06, 1, 0xab)), Ok(true));
    assert_eq!(touch::read(&mut report(0x06, 0x80, 0xab)), Ok(false));
    assert_eq!(touch::read(&mut report(0x00, 1, 0xab)), Ok(false));
    assert_eq!(touch::read(&mut report(0x06, 1, 0x00)), Err(Error::Invalid));
    assert_eq!(touch::read(&mut Controller(None)), Err(Error::Bus));
}

#[test]
fn probe_checks_communication_only() {
    assert_eq!(touch::probe(&mut report(0, 0, 0)), Ok(()));
    assert_eq!(touch::probe(&mut Controller(None)), Err(Error::Bus));
}
