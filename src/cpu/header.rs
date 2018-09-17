use cpu;
use std::fmt;
use std::str;

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

pub struct Header {
    nintendo: Vec<u8>,
    title: String,
    manufacturer: u32,
    gcb: bool,
    licensee: String,
    sgb: bool,
    cartrigte_type: String,
    rom_size: u8,
    ram_size: u8,
    destination_code: bool,
    rom_version: u8,
    header_checksum: u8,
    global_checksum: u8,
}

impl Header {
    pub fn new(bytes: &Vec<u8>) -> Header {
        Header {
            nintendo: bytes[NINTENDO.0..(NINTENDO.1 + 1)].to_vec(),
            title: str::from_utf8(&bytes[TITLE.0..(TITLE.1)])
                .unwrap()
                .to_string(),
            manufacturer: cpu::bytes_to_u32(&bytes[MANUFACTURER.0..(MANUFACTURER.1 + 1)]),
            gcb: bytes[GCB.0] != 0,
            licensee: decode_license(&bytes),
            sgb: bytes[SGB.0] != 0,
            cartrigte_type: decode_cartrigte_type(bytes[CARTRIDGE_TYPE.0]),
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
fn decode_license(bytes: &Vec<u8>) -> String {
    match cpu::bytes_to_u16(&bytes[LICENSEE.0..(LICENSEE.1 + 1)]) {
        0x00 => "None".to_string(),
        0x01 => "Nintendo".to_string(),
        0x33 => decode_extended_license(&bytes),
        code => format!("Other: 0x{:x}", code),
    }
}

///! Decode extended license code.
/// TODO(slongfield): The decode here seems to be missing some documentation, figure out the
/// algorithm and document. This works for now, though.
fn decode_extended_license(bytes: &Vec<u8>) -> String {
    match cpu::bytes_to_u16(&bytes[NEW_LICENSEE.0..(NEW_LICENSEE.1 + 1)]) {
        0x3031 => "Nintendo (new)".to_string(),
        code => format!("Other Extended License: 0x{:x}", code),
    }
}

///! Decodes the type of the cartrigte.
/// TODO(slongfield): This probably makes more sense, and is more useful, as an enum.
fn decode_cartrigte_type(byte: u8) -> String {
    match byte {
        0x00 => "ROM ONLY".to_string(),
        0x01 => "MBC1".to_string(),
        0x02 => "MBC1+RAM".to_string(),
        0x13 => "MBC3+RAM+BATTERY".to_string(),
        0x1B => "MBC5+RAM+BATTERY".to_string(),
        code => format!("Unknown cartrigte type: 0x{:x}", code),
    }
}

///! The Nintendo logo is stored in memory as tiles. Special-purpose renderer here, so it can run without any other dependencies.
pub fn draw_nintendo(bytes: &Vec<u8>, f: &mut fmt::Formatter) -> fmt::Result {
    let mut bits: Vec<bool> = vec![];
    for byte in bytes.iter() {
        for mask in BIT_MASKS.iter() {
            bits.push(byte & mask != 0);
        }
    }
    let mut bitmap = [[false; 48]; 8];
    let mut bit_pos: usize = 0;
    for y_base in [0, 4].iter() {
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
    write!(
        f,
        "\n{}\n",
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
                }).collect::<String>()).collect::<Vec<String>>()
            .join("\n")
    );

    Ok(())
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        draw_nintendo(&self.nintendo, f)?;
        write!(f, "\nTitle: {}\n", self.title)?;
        write!(f, "Manufacturer Code: 0x{:x}\n", self.manufacturer)?;
        write!(f, "Gameboy Color Game: {}\n", self.gcb)?;
        write!(f, "Licensee: {}\n", self.licensee)?;
        write!(f, "Super Gameboy Support: {}\n", self.sgb)?;
        write!(f, "Cartridge type: {}\n", self.cartrigte_type)?;
        write!(f, "ROM size: 0x{:02x}\n", self.rom_size)?;
        write!(f, "RAM size: 0x{:02x}\n", self.ram_size)?;
        write!(f, "ROM version: 0x{:02x}\n", self.rom_version)?;
        write!(f, "Japan-only?: {}\n", self.destination_code)?;
        write!(f, "Header checksum: 0x{:02x}\n", self.header_checksum)?;
        write!(f, "Global checksum: 0x{:02x}\n", self.global_checksum)
    }
}
