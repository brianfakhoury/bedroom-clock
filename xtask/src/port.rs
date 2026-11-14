//! Find the clock's serial port.

use serialport::{SerialPortType, UsbPortInfo};

const ESPRESSIF: u16 = 0x303a;
const USB_SERIAL_JTAG: u16 = 0x1001;

#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub name: String,
    pub usb: UsbPortInfo,
}

/// The connected Espressif USB serial ports
pub fn connected() -> Result<Vec<Port>, serialport::Error> {
    let ports =
        serialport::available_ports()?.into_iter().filter_map(|port| match port.port_type {
            // macOS lists each device twice; the /dev/cu. name is the one to open
            SerialPortType::UsbPort(usb)
                if usb.vid == ESPRESSIF && !port.port_name.starts_with("/dev/tty.") =>
            {
                Some(Port { name: port.port_name, usb })
            }
            _ => None,
        });
    Ok(ports.collect())
}

/// Use the requested port, or the only connected clock
pub fn select(requested: Option<&str>, connected: &[Port]) -> Result<Port, String> {
    match (requested, connected) {
        (Some(name), _) => Ok(connected.iter().find(|port| port.name == name).cloned().unwrap_or(
            // Trust an explicit choice, as the board's built-in USB port
            Port {
                name: name.to_owned(),
                usb: UsbPortInfo {
                    vid: ESPRESSIF,
                    pid: USB_SERIAL_JTAG,
                    serial_number: None,
                    manufacturer: None,
                    product: None,
                },
            },
        )),
        (None, [port]) => Ok(port.clone()),
        (None, []) => Err("Connect USB, tap the screen, and press KEY to keep it awake.".into()),
        (None, ports) => {
            let names: Vec<&str> = ports.iter().map(|port| port.name.as_str()).collect();
            Err(format!("Multiple devices; choose one with --port: {}", names.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(name: &str) -> Port {
        select(Some(name), &[]).unwrap()
    }

    #[test]
    fn selection_requires_one_connected_clock() {
        let clock = port("clock");
        assert_eq!(select(None, std::slice::from_ref(&clock)), Ok(clock.clone()));
        assert!(select(None, &[]).is_err());
        let error = select(None, &[clock.clone(), port("other")]).unwrap_err();
        assert!(error.contains("clock, other"));
        // An explicit choice needs no detection
        assert_eq!(select(Some("clock"), &[port("other")]), Ok(clock));
    }
}
