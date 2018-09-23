///! Generic helpers for manipulating bytes.

// TODO(slongfield): These should probably be templates of some form, and 'util' is a dumb
// name for a module.

pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut outp: u32 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u32::from(*byte);
    }
    outp
}

pub fn bytes_to_u16(bytes: &[u8]) -> u16 {
    let mut outp: u16 = 0;
    for byte in bytes {
        outp <<= 8;
        outp |= u16::from(*byte);
    }
    outp
}
