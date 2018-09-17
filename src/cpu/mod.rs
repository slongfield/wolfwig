pub mod cpu;
pub mod decode;
pub mod header;
pub mod registers;

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut outp: u32 = 0;
    for byte in bytes {
        outp = outp << 8;
        outp |= *byte as u32;
    }
    outp
}

fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut outp: u16 = 0;
    for byte in bytes {
        outp = outp << 8;
        outp |= *byte as u16;
    }
    outp
}
