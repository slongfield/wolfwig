use std::fmt;
use std::str;
use util;

///! Constants associated with the ROM header. Each of these is a range of bytes in the header.
const NINTENDO: (usize, usize) = (0x0104, 0x0133);
const TITLE: (usize, usize) = (0x0134, 0x0143);
const MANUFACTURER: (usize, usize) = (0x013F, 0x0142);
const GCB: (usize, usize) = (0x0143, 0x0143);
const NEW_LICENSEE: (usize, usize) = (0x0144, 0x0145);
const SGB: (usize, usize) = (0x0146, 0x0146);
const CARTRIDGE_TYPE: (usize, usize) = (0x0147, 0x0147);
const ROM_SIZE: (usize, usize) = (0x0148, 0x0148);
const RAM_SIZE: (usize, usize) = (0x0149, 0x0149);
const DESTINATION_CODE: (usize, usize) = (0x014A, 0x014A);
const LICENSEE: (usize, usize) = (0x014B, 0x014B);
const ROM_VERSION: (usize, usize) = (0x014C, 0x014C);
const HEADER_CHECKSUM: (usize, usize) = (0x014D, 0x014D);
const GLOBAL_CHECKSUM: (usize, usize) = (0x014E, 0x014E);
const BIT_MASKS: [u8; 8] = [1 << 7, 1 << 6, 1 << 5, 1 << 4, 1 << 3, 1 << 2, 1 << 1, 1];

#[derive(Debug)]
pub enum CartridgeType {
    Rom,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    Mmm01,
    Mmm01Ram,
    Mmm01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerBatteryRam,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    Mbc6,
    Mbc7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

pub struct Header {
    nintendo: Vec<u8>,
    title: String,
    manufacturer: u32,
    gcb: bool,
    licensee: String,
    sgb: bool,
    pub cartridge_type: CartridgeType,
    rom_size: u8,
    ram_size: u8,
    destination_code: bool,
    rom_version: u8,
    header_checksum: u8,
    global_checksum: u8,
}

impl Header {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            nintendo: bytes[NINTENDO.0..(NINTENDO.1 + 1)].to_vec(),
            title: str::from_utf8(&bytes[TITLE.0..(TITLE.1)])
                .unwrap()
                .to_string(),
            manufacturer: util::bytes_to_u32(&bytes[MANUFACTURER.0..(MANUFACTURER.1 + 1)]),
            gcb: bytes[GCB.0] != 0,
            licensee: decode_license(&bytes),
            sgb: bytes[SGB.0] != 0,
            cartridge_type: decode_cartridge_type(bytes[CARTRIDGE_TYPE.0]),
            rom_size: bytes[ROM_SIZE.0],
            ram_size: bytes[RAM_SIZE.0],
            destination_code: bytes[DESTINATION_CODE.0] == 0,
            rom_version: bytes[ROM_VERSION.0],
            // TODO(slongfield): Verify checksum validity.
            header_checksum: bytes[HEADER_CHECKSUM.0],
            global_checksum: bytes[GLOBAL_CHECKSUM.0],
        }
    }
}

///! Decodes the licensee codes.
/// TODO(slongfield): Transcribe the full list.
fn decode_license(bytes: &[u8]) -> String {
    match util::bytes_to_u16(&bytes[LICENSEE.0..(LICENSEE.1 + 1)]) {
        0x00 => "None".to_string(),
        0x01 => "Nintendo".to_string(),
        0x33 => decode_extended_license(&bytes),
        code => format!("Other: 0x{:x}", code),
    }
}

///! Decode extended license code.
/// TODO(slongfield): The decode here seems to be missing some documentation, figure out the
/// algorithm and document. This works for now, though.
fn decode_extended_license(bytes: &[u8]) -> String {
    match util::bytes_to_u16(&bytes[NEW_LICENSEE.0..(NEW_LICENSEE.1 + 1)]) {
        0x3031 => "Nintendo (new)".to_string(),
        code => format!("Other Extended License: 0x{:x}", code),
    }
}

///! Decodes the type of the cartrigte.
fn decode_cartridge_type(byte: u8) -> CartridgeType {
    match byte {
        0x00 => CartridgeType::Rom,
        0x01 => CartridgeType::Mbc1,
        0x02 => CartridgeType::Mbc1Ram,
        0x03 => CartridgeType::Mbc1RamBattery,
        0x05 => CartridgeType::Mbc2,
        0x06 => CartridgeType::Mbc2Battery,
        0x08 => CartridgeType::RomRam,
        0x09 => CartridgeType::RomRamBattery,
        0x0B => CartridgeType::Mmm01,
        0x0C => CartridgeType::Mmm01Ram,
        0x0D => CartridgeType::Mmm01RamBattery,
        0x0F => CartridgeType::Mbc3TimerBattery,
        0x10 => CartridgeType::Mbc3TimerBatteryRam,
        0x11 => CartridgeType::Mbc3,
        0x12 => CartridgeType::Mbc3Ram,
        0x13 => CartridgeType::Mbc3RamBattery,
        0x19 => CartridgeType::Mbc5,
        0x1A => CartridgeType::Mbc5Ram,
        0x1B => CartridgeType::Mbc5RamBattery,
        0x1C => CartridgeType::Mbc5Rumble,
        0x1D => CartridgeType::Mbc5RumbleRam,
        0x1E => CartridgeType::Mbc5RumbleRamBattery,
        0x20 => CartridgeType::Mbc6,
        0x22 => CartridgeType::Mbc7SensorRumbleRamBattery,
        0xFC => CartridgeType::PocketCamera,
        0xFD => CartridgeType::BandaiTama5,
        0xFE => CartridgeType::HuC3,
        0xFF => CartridgeType::HuC1RamBattery,
        code => panic!("Unknown cartrigte type: 0x{:x}", code),
    }
}

///! The Nintendo logo is stored in memory as pixel tiles, but compressed relative to normal
///! tiles, representing each 4-pixel chunk with one bit.
pub fn draw_nintendo(bytes: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    let mut bits: Vec<bool> = vec![];
    for byte in bytes.iter() {
        for mask in &BIT_MASKS {
            bits.push(byte & mask != 0);
        }
    }
    let mut bitmap = [[false; 48]; 8];
    let mut bit_pos: usize = 0;
    for y_base in &[0, 4] {
        for x_base in 0..12 {
            for y_offset in 0..4 {
                for x_offset in 0..4 {
                    if bit_pos == bits.len() {
                        continue;
                    }
                    bitmap[(y_base + y_offset) as usize][(x_base * 4 + x_offset) as usize] =
                        bits[bit_pos];
                    bit_pos += 1;
                }
            }
        }
    }
    writeln!(
        f,
        "\n{}",
        bitmap
            .chunks(2)
            .map(|rows| rows[0]
                .iter()
                .zip(rows[1].iter())
                .map(|(t, b)| match (t, b) {
                    (false, false) => " ",
                    (true, false) => "▀",
                    (false, true) => "▄",
                    (true, true) => "█",
                })
                .collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    )?;

    Ok(())
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        draw_nintendo(&self.nintendo, f)?;
        writeln!(f, "\nTitle: {}", self.title)?;
        writeln!(f, "Manufacturer Code: 0x{:x}", self.manufacturer)?;
        writeln!(f, "Gameboy Color Game: {}", self.gcb)?;
        writeln!(f, "Licensee: {}", self.licensee)?;
        writeln!(f, "Super Gameboy Support: {}", self.sgb)?;
        writeln!(f, "Cartridge type: {:?}", self.cartridge_type)?;
        writeln!(f, "ROM size: 0x{:02x}", self.rom_size)?;
        writeln!(f, "RAM size: 0x{:02x}", self.ram_size)?;
        writeln!(f, "ROM version: 0x{:02x}", self.rom_version)?;
        writeln!(f, "Japan-only?: {}", self.destination_code)?;
        writeln!(f, "Header checksum: 0x{:02x}", self.header_checksum)?;
        writeln!(f, "Global checksum: 0x{:02x}", self.global_checksum)
    }
}
