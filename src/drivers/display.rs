//! CO5300 AMOLED panel.

use crate::hal::qspi::Qspi;
use crate::hal::timer::Delay;

const DISPLAY_OFF: u8 = 0x28;
const DISPLAY_ON: u8 = 0x29;
const SLEEP_OUT: u8 = 0x11;
const SLEEP_IN: u8 = 0x10;
const COLUMN_ADDRESS: u8 = 0x2a;
const ROW_ADDRESS: u8 = 0x2b;
const MEMORY_WRITE: u8 = 0x2c;
const MEMORY_CONTINUE: u8 = 0x3c;
const BRIGHTNESS: u8 = 0x51;
const SPI_COMMAND: u8 = 0x02;
const QSPI_PIXELS: u8 = 0x32;

pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 480;

/// A pixel window. CO5300 requires even x, y, width and height, and nonzero
/// dimensions
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// What the application draws on. Colors are RGB565
pub trait Panel {
    fn fill(&mut self, area: Rect, color: u16);
    /// The pixel function receives screen coordinates
    fn pattern(&mut self, area: Rect, pixel: impl Fn(u32, u32) -> u16);
    fn on(&mut self);
    fn off(&mut self);
    fn brightness(&mut self, value: u8);
    /// Sleep retains pixels
    fn sleep(&mut self);
    /// Wake restores panel settings
    fn wake(&mut self);
}

pub struct Display<Q, D> {
    qspi: Q,
    delay: D,
    sleeping: bool,
}

impl<Q: Qspi, D: Delay> Display<Q, D> {
    pub const fn new(qspi: Q, delay: D) -> Self {
        Self { qspi, delay, sleeping: false }
    }

    /// The bus, for the board to configure its pins
    pub fn bus(&self) -> &Q {
        &self.qspi
    }

    /// Initialize after the board configures the bus pins and resets the panel
    #[cfg_attr(target_os = "none", unsafe(link_section = ".init.code"))]
    #[inline(never)]
    pub fn init(&mut self) {
        self.command(DISPLAY_OFF);
        self.command(SLEEP_OUT);
        self.delay.delay_us(600_000);
        self.configure();
        self.fill(Rect::new(0, 0, WIDTH, HEIGHT), 0);
        self.off();
    }

    fn write_command(&mut self, code: u8, data: &[u8]) {
        self.qspi.begin();
        for byte in [SPI_COMMAND, 0, code, 0] {
            self.qspi.write_byte(byte);
        }
        for &byte in data {
            self.qspi.write_byte(byte);
        }
        self.qspi.end();
    }

    fn command(&mut self, code: u8) {
        self.write_command(code, &[]);
    }

    fn write_register(&mut self, code: u8, value: u8) {
        self.write_command(code, &[value]);
    }

    fn draw(&mut self, area: Rect, pixel: impl Fn(u32, u32) -> u16) {
        let Rect { x, y, width, height } = area;
        let (right, bottom) = (x + width - 1, y + height - 1);
        self.write_command(
            COLUMN_ADDRESS,
            &[(x >> 8) as u8, x as u8, (right >> 8) as u8, right as u8],
        );
        self.write_command(
            ROW_ADDRESS,
            &[(y >> 8) as u8, y as u8, (bottom >> 8) as u8, bottom as u8],
        );
        self.command(MEMORY_WRITE);
        self.qspi.begin();
        for byte in [QSPI_PIXELS, 0, MEMORY_CONTINUE, 0] {
            self.qspi.write_byte(byte);
        }
        for row in y..=bottom {
            for column in x..=right {
                self.qspi.write_quad16(pixel(column, row));
            }
        }
        self.qspi.end();
    }

    fn configure(&mut self) {
        self.write_register(0xfe, 0x20);
        self.write_register(0x19, 0x10);
        self.write_register(0x1c, 0xa0);
        self.write_register(0xfe, 0);
        self.write_register(0xc4, 0x80);
        self.write_register(0x3a, 0x55);
        self.write_register(0x36, 0x30);
        self.write_register(0x53, 0x20);
        self.delay.delay_us(10_000);
        self.write_register(0x63, 0xff);
        self.delay.delay_us(10_000);
        self.write_register(BRIGHTNESS, 0);
        self.write_register(0x58, 0);
    }
}

impl<Q: Qspi, D: Delay> Panel for Display<Q, D> {
    fn fill(&mut self, area: Rect, color: u16) {
        self.draw(area, |_, _| color);
    }

    fn pattern(&mut self, area: Rect, pixel: impl Fn(u32, u32) -> u16) {
        self.draw(area, pixel);
    }

    fn on(&mut self) {
        self.command(DISPLAY_ON);
    }

    fn off(&mut self) {
        self.command(DISPLAY_OFF);
    }

    fn brightness(&mut self, value: u8) {
        self.write_register(BRIGHTNESS, value);
    }

    fn sleep(&mut self) {
        if self.sleeping {
            return;
        }
        self.command(DISPLAY_OFF);
        self.command(SLEEP_IN);
        // CO5300 pp.143–145: opposite sleep transitions must be 120 ms apart
        // Complete the guard before CPU sleep, including immediate retaps
        self.delay.delay_us(120_000);
        self.sleeping = true;
    }

    fn wake(&mut self) {
        if !self.sleeping {
            return;
        }
        self.command(SLEEP_OUT);
        self.delay.delay_us(120_000);
        // Sleep Out reloads extended/test defaults. Restore the module settings
        // while dark; no reset or framebuffer clear, so retained pixels survive
        self.configure();
        self.sleeping = false;
    }
}
