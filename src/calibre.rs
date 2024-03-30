use libc::c_char;

#[link(name = "palmdoc")]
extern "C" {
    fn cpalmdoc_decompress(input: *const c_char, input_len: usize, output: *mut c_char) -> usize;

    fn cpalmdoc_compress(input: *const c_char, input_len: usize, output: *mut c_char) -> usize;
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut output = vec![0; input.len() * 8];
    let input_len = input.len();

    unsafe {
        let decompressed_len = cpalmdoc_decompress(
            input.as_ptr() as *const c_char,
            input_len,
            output.as_mut_ptr() as *mut c_char,
        );

        output.truncate(decompressed_len);
        output
    }
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut output = vec![0; input.len() * 2];
    let input_len = input.len();

    unsafe {
        let compressed_len = cpalmdoc_compress(
            input.as_ptr() as *const c_char,
            input_len,
            output.as_mut_ptr() as *mut c_char,
        );

        output.truncate(compressed_len);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lipsum::lipsum;

    #[test]
    fn test_roundtrip() {
        let input = lipsum(4096);
        let input = input.as_bytes()[..4096].to_vec();
        let compressed = compress(&input);
        let decompressed = decompress(&compressed);

        assert_eq!(input, decompressed);
    }
}
