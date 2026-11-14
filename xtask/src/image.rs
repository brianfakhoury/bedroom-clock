//! Convert the firmware ELF to the image that the ESP32-C6 ROM loads from the
//! start of flash, byte for byte as `esptool elf2image` writes it.

use object::Endianness;
use object::elf::{FileHeader32, SHT_NOBITS, SHT_PROGBITS};
use object::read::elf::{FileHeader, SectionHeader};
use sha2::{Digest, Sha256};

const MAGIC: u8 = 0xe9;
const FLASH_MODE_DIO: u8 = 2;
/// 16 MB in the high nibble, 40 MHz in the low
const FLASH_SIZE_AND_FREQUENCY: u8 = 0x40;
const CHIP_ESP32C6: u16 = 13;
const CHECKSUM_SEED: u8 = 0xef;

/// A section that occupies memory
#[derive(Debug)]
struct Section<'elf> {
    name: &'elf [u8],
    address: u32,
    size: u32,
    /// Empty for sections that occupy no file space, such as `.bss`
    data: &'elf [u8],
}

fn sections(elf: &[u8]) -> Result<(u32, Vec<Section<'_>>), object::Error> {
    let header = FileHeader32::<Endianness>::parse(elf)?;
    let endian = header.endian()?;
    let table = header.sections(endian, elf)?;
    let mut sections = Vec::new();
    for section in table.iter() {
        let kind = section.sh_type(endian);
        if (kind == SHT_PROGBITS || kind == SHT_NOBITS) && section.sh_addr(endian) != 0 {
            sections.push(Section {
                name: table.section_name(endian, section)?,
                address: section.sh_addr(endian),
                size: section.sh_size(endian),
                data: section.data(endian, elf)?,
            });
        }
    }
    sections.sort_by_key(|section| section.address);
    Ok((header.e_entry(endian), sections))
}

pub fn from_elf(elf: &[u8]) -> Result<Vec<u8>, object::Error> {
    let (entry, sections) = sections(elf)?;
    Ok(image(entry, &segments(&sections)))
}

/// Load address and contents of each run of touching sections
fn segments(sections: &[Section]) -> Vec<(u32, Vec<u8>)> {
    let mut segments: Vec<(u32, Vec<u8>)> = Vec::new();
    for section in sections.iter().filter(|section| !section.data.is_empty()) {
        // Pad each section to a whole word, which closes alignment gaps
        let mut data = section.data.to_vec();
        data.resize(data.len().next_multiple_of(4), 0);
        match segments.last_mut() {
            Some((address, joined)) if *address + joined.len() as u32 == section.address => {
                joined.extend(data);
            }
            _ => segments.push((section.address, data)),
        }
    }
    segments
}

fn image(entry: u32, segments: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut image = vec![MAGIC, segments.len() as u8, FLASH_MODE_DIO, FLASH_SIZE_AND_FREQUENCY];
    image.extend(entry.to_le_bytes());
    image.extend([0xee, 0, 0, 0]); // No write-protect pin; default drive strengths
    image.extend(CHIP_ESP32C6.to_le_bytes());
    image.extend([0, 0, 0]); // Any chip revision from 0.0
    image.extend(u16::MAX.to_le_bytes()); // No maximum revision
    image.extend([0, 0, 0, 0, 1]); // Reserved; a SHA-256 digest follows the image
    for (address, data) in segments {
        image.extend(address.to_le_bytes());
        image.extend((data.len() as u32).to_le_bytes());
        image.extend(data);
    }

    // The checksum of all segment data ends a 16-byte block
    image.resize((image.len() + 1).next_multiple_of(16) - 1, 0);
    let data = segments.iter().flat_map(|(_, data)| data);
    image.push(data.fold(CHECKSUM_SEED, |checksum, byte| checksum ^ byte));
    let digest = Sha256::digest(&image);
    image.extend(digest);
    image
}

/// Bytes each part of the firmware occupies
#[derive(Default, Debug, PartialEq)]
pub struct Sizes {
    pub init: u32,
    pub text: u32,
    pub rodata: u32,
    pub data: u32,
    pub bss: u32,
    /// LP RAM that survives sleep, including alignment between sections
    pub retained: u32,
}

const LP_RAM: u32 = 0x5000_0000;

pub fn sizes(elf: &[u8]) -> Result<Sizes, object::Error> {
    let mut sizes = Sizes::default();
    for section in sections(elf)?.1 {
        let size = match section.name {
            b".init" => &mut sizes.init,
            b".text" => &mut sizes.text,
            b".rodata" => &mut sizes.rodata,
            b".data" => &mut sizes.data,
            b".bss" => &mut sizes.bss,
            _ => continue,
        };
        *size = section.size;
        if section.address >= LP_RAM {
            sizes.retained = section.address + section.size - LP_RAM;
        }
    }
    Ok(sizes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(address: u32, data: &[u8]) -> Section<'_> {
        Section { name: b"", address, size: data.len() as u32, data }
    }

    #[test]
    fn touching_sections_join_after_padding() {
        let sections = [
            section(0x4080_0000, &[1, 2, 3, 4, 5, 6]),
            section(0x4080_0008, &[7]),
            section(0x5000_0000, &[8, 9, 10, 11]),
            section(0x5000_0008, &[12]),
            section(0x5000_000c, &[]),
        ];
        let expected = [
            (0x4080_0000, vec![1, 2, 3, 4, 5, 6, 0, 0, 7, 0, 0, 0]),
            (0x5000_0000, vec![8, 9, 10, 11]),
            (0x5000_0008, vec![12, 0, 0, 0]),
        ];
        assert_eq!(segments(&sections), expected);
    }

    #[test]
    fn image_layout() {
        let image =
            image(0x5000_0020, &[(0x4080_0000, vec![1, 2, 3, 4]), (0x5000_0000, vec![0x10; 8])]);
        let header = [
            [0xe9, 2, 2, 0x40, 0x20, 0, 0, 0x50],
            [0xee, 0, 0, 0, 13, 0, 0, 0],
            [0, 0xff, 0xff, 0, 0, 0, 0, 1],
        ];
        assert_eq!(image[..24], header.concat());
        assert_eq!(image[24..36], [0, 0, 0x80, 0x40, 4, 0, 0, 0, 1, 2, 3, 4]);
        assert_eq!(image[36..44], [0, 0, 0, 0x50, 8, 0, 0, 0]);
        assert_eq!(image[44..52], [0x10; 8]);
        // Zero padding, then the checksum as the last byte of a 16-byte block
        assert_eq!(image[52..63], [0; 11]);
        assert_eq!(image[63], 0xef ^ 1 ^ 2 ^ 3 ^ 4);
        assert_eq!(image[64..], Sha256::digest(&image[..64])[..]);
    }
}
