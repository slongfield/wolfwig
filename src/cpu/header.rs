use std::fmt;

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

pub struct Header {
    nintendo: Vec<u8>,
    title: String,
    manufacturer: u32,
    gcb: bool,
    licensee: String,
    sgb: bool,
    cartrigte_type: String,
    // TODO(slongfield): These might be more useful as u32. str for now for ease of Display.
    rom_size: String,
    ram_size: String,
    destination_code: bool,
    rom_version: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl Header {
    pub fn new(bytes: &Vec<u8>) -> Header {
        Header {
            nintendo: bytes[NINTENDO.0..(NINTENDO.1 + 1)].to_vec(),
            title: "".to_string(),
            manufacturer: 0,
            gcb: false,
            licensee: "".to_string(),
            sgb: false,
            cartrigte_type: "".to_string(),
            rom_size: "0".to_string(),
            ram_size: "0".to_string(),
            destination_code: false,
            rom_version: 0,
            header_checksum: 0,
            global_checksum: 0,
        }
    }
}

///! The Nintendo logo is stored in memory as tiles. Special-purpose renderer here, so it can run without any other dependencies.
pub fn draw_nintendo(bytes: &Vec<u8>, f: &mut fmt::Formatter) -> fmt::Result {
    let mut bits: Vec<bool> = vec![];
    let bit_masks = vec![1 << 7, 1 << 6, 1 << 5, 1 << 4, 1 << 3, 1 << 2, 1 << 1, 1];
    for (index, byte) in bytes.iter().enumerate() {
        for mask in bit_masks.iter() {
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
        write!(f, "Nintendo header:\n")?;
        draw_nintendo(&self.nintendo, f)
    }
}
