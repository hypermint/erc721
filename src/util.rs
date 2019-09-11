pub fn make_key_by_parts(parts: Vec<&[u8]>) -> Vec<u8> {
    parts.join(&b'/')
}
