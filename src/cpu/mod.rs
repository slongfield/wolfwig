pub mod decode;
pub mod header;
pub mod lr25902;
pub mod registers;

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut outp: u32 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u32::from(*byte);
    }
    outp
}

fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut outp: u16 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u16::from(*byte);
    }
    outp
}
