#[inline]
pub fn atou8(c: u8) -> u8 {
    if c <= b'9' {
        c - 48
    } else if c <= b'F' {
        c - 55
    } else if c <= b'f' {
        c - 87
    } else {
        0
    }
}

#[inline]
pub fn atou16(c: u8) -> u16 {
    atou8(c) as u16
}

#[inline]
pub fn hex_to_u8(bytes: &[u8]) -> u8 {
    16 * atou8(bytes[0]) + atou8(bytes[1])
}

#[inline]
pub fn hex_to_u16(bytes: &[u8]) -> u16 {
    4096 * atou16(bytes[0])
        + 256 * atou16(bytes[1])
        + 16 * atou16(bytes[2])
        + atou16(bytes[3])
}
