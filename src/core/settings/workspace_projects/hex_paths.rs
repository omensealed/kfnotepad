fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push(hex_digit(byte >> 4));
        hex.push(hex_digit(byte & 0x0f));
    }
    hex
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks_exact(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Some(bytes)
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("hex digit nibble must be below 16"),
    }
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(unix)]
pub(crate) fn path_to_hex(path: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;

    bytes_to_hex(path.as_os_str().as_bytes())
}

#[cfg(unix)]
pub(crate) fn path_from_hex(hex: &str) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    Some(PathBuf::from(OsString::from_vec(hex_to_bytes(hex)?)))
}

#[cfg(not(unix))]
pub(crate) fn path_to_hex(path: &Path) -> String {
    bytes_to_hex(path.to_string_lossy().as_bytes())
}

#[cfg(not(unix))]
pub(crate) fn path_from_hex(hex: &str) -> Option<PathBuf> {
    String::from_utf8(hex_to_bytes(hex)?)
        .ok()
        .map(PathBuf::from)
}
