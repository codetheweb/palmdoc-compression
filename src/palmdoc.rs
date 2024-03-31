use std::{collections::HashMap, ops::Range};

const BASE: u64 = 257; // Prime base of polynomial rolling hash
const MODULUS: u64 = 1_000_000_007;

struct RollingHash {
    hash: u64,
    base_pow_n: u64, // BASE^window_size mod MODULUS
}

impl RollingHash {
    fn new(window: &[u8]) -> Self {
        let mut hash = 0;
        let mut base_pow_n = 1;
        for &byte in window.iter().rev() {
            hash = (hash * BASE + byte as u64) % MODULUS;
            if base_pow_n < window.len() as u64 {
                base_pow_n = (base_pow_n * BASE) % MODULUS;
            }
        }
        Self { hash, base_pow_n }
    }

    fn roll(&mut self, out_byte: u8, in_byte: u8) {
        let out_contribution = (out_byte as u64 * self.base_pow_n) % MODULUS;
        self.hash = (self.hash + MODULUS - out_contribution) % MODULUS; // Remove old byte
        self.hash = (self.hash * BASE + in_byte as u64) % MODULUS; // Add new byte
    }
}

pub fn compress_palmdoc(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    let len = data.len();

    let mut hash_table: HashMap<u64, Vec<usize>> = HashMap::new();
    let window_size = 3; // Example window size
    for i in 0..data.len() - window_size + 1 {
        let window = &data[i..i + window_size];
        let rolling_hash = RollingHash::new(window);
        hash_table.entry(rolling_hash.hash).or_default().push(i);
    }

    while i < len {
        if i > 10 && (len - i) > 10 {
            let mut match_range: Option<Range<usize>> = None;

            let chunk = &data[i..(i + window_size)];

            let current_hash = RollingHash::new(chunk);
            if let Some(positions) = hash_table.get(&current_hash.hash) {
                for &position in positions {
                    if position >= i || i - position > 2047 {
                        continue;
                    }

                    for j in (window_size..11).rev() {
                        if j > match_range.as_ref().unwrap_or(&(0..0)).len()
                            && data[position..position + j] == data[i..i + j]
                        {
                            match_range = Some(position..position + j);
                            break;
                        }
                    }
                }
            }

            if let Some(match_range) = match_range {
                let m = (i - match_range.start) as u16;
                let code = 0x8000 + ((m << 3) & 0x3ff8) + ((match_range.len() as u16) - 3);
                out.extend(&code.to_be_bytes());
                i += match_range.len();
                continue;
            }
        }

        // Single byte encoding or special cases handling
        let byte = data[i];
        i += 1;

        if byte == b' ' && i + 1 < len && (0x40..0x80).contains(&data[i]) {
            out.push(data[i] ^ 0x80);
            i += 1;
            continue;
        }

        if byte == 0 || (byte > 8 && byte < 0x80) {
            out.push(byte);
        } else {
            let mut j = i;
            let mut binseq = vec![byte];

            while j < len && binseq.len() < 8 {
                let ch = data[j];
                if ch == 0 || (ch > 8 && ch < 0x80) {
                    break;
                }

                binseq.push(ch);
                j += 1;
            }

            out.extend(&(binseq.len() as u8).to_be_bytes());
            out.extend(&binseq);
            i += binseq.len() - 1;
        }
    }

    out
}

pub fn decompress_palmdoc(data: &[u8]) -> Vec<u8> {
    // Adapted from https://metacpan.org/release/AZED/EBook-Tools-0.3.3/source/lib/EBook/Tools/PalmDoc.pm
    let len = data.len();
    let mut offset = 0;
    let mut uncompressed = Vec::new();

    while offset < len {
        let byte = data[offset];
        offset += 1;

        if byte == 0 {
            // Nulls are literal
            uncompressed.push(byte);
        } else if byte <= 8 {
            // Next byte is literal
            uncompressed.extend_from_slice(&data[offset..offset + byte as usize]);
            offset += byte as usize;
        } else if byte <= 0x7f {
            // Values from 0x09 through 0x7f are literal
            uncompressed.push(byte);
        } else if byte <= 0xbf {
            // Data is LZ77-compressed
            offset += 1;

            if offset > len {
                println!("WARNING: offset to LZ77 bits is outside of the data");
                return Vec::new();
            }

            let mut lz77 = u16::from_be_bytes([data[offset - 2], data[offset - 1]]);

            // Leftmost two bits are ID bits and need to be dropped
            lz77 &= 0x3fff;

            // Length is rightmost 3 bits + 3
            let lz77length = ((lz77 & 0x0007) as usize) + 3;

            // Remaining 11 bits are offset
            let lz77offset = (lz77 >> 3) as usize;

            if lz77offset < 1 {
                println!("WARNING: LZ77 decompression offset is invalid!");
                return Vec::new();
            }

            // Getting text from the offset
            let mut textlength = uncompressed.len();
            for _ in 0..lz77length {
                let textpos = textlength - lz77offset;
                uncompressed.push(uncompressed[textpos]);
                textlength += 1;
            }
        } else {
            // 0xc0 - 0xff are single characters (XOR 0x80) preceded by a space
            uncompressed.push(b' ');
            uncompressed.push(byte ^ 0x80);
        }
    }
    uncompressed
}

#[cfg(test)]
mod tests {
    use lipsum::lipsum;

    use super::*;

    fn get_calibre_testcases() -> Vec<(Vec<u8>, Vec<u8>)> {
        // Test cases taken from Calibre
        // (input, compressed_result)
        return vec![
            (
                hex::decode("616263030405066d73").unwrap(),
                hex::decode("61626304030405066d73").unwrap(),
            ),
            (
                hex::decode("612062206320fe6420").unwrap(),
                hex::decode("61e2e32001fe6420").unwrap(),
            ),
            (
                hex::decode("303132333435363738396178797a326278797a3263646667666f39697579657268")
                    .unwrap(),
                hex::decode("303132333435363738396178797a3262802963646667666f39697579657268")
                    .unwrap(),
            ),
            (
              hex::decode("30313233343536373839617364303132333435363738396173647c79797a7a7878666668686a6a6b6b").unwrap(),
              hex::decode("30313233343536373839617364806f80687c79797a7a7878666668686a6a6b6b").unwrap()
            ),
            (
              hex::decode("6369657761636e6171206569753734332072373837712030772520203b207361206664ef0c6664786f73616320776f636a702061636f6965636f776569206f77616963206a6f63696f7761706a636976636a706f69766a706f726569766a706f617663613b207039617738373433793672373425245e245e253820").unwrap(),
              hex::decode("6369657761636e6171e56975373433f2373837712030772520203bf361e66401ef0c6664786f736163f76f636a70e1636f6965636f776569ef77616963ea6f63698050706a63697681086f697680287265803a617663613bf03961773882f0793672373425245e245e253820").unwrap()
            ),
            (
                hex::decode("61626373646661736466616263646173646f66617373").unwrap(),
                hex::decode("61626373646661736466616263646173646f66617373").unwrap()
            )
        ];
    }

    #[test]
    fn test_compress_palmdoc() {
        for (input, expected) in get_calibre_testcases() {
            let compressed = compress_palmdoc(&input);
            assert_eq!(compressed, expected);
        }
    }

    #[test]
    fn test_decompress_palmdoc() {
        for (expected, compressed) in get_calibre_testcases() {
            let decompressed = decompress_palmdoc(&compressed);
            assert_eq!(decompressed, expected);
        }
    }

    #[test]
    fn test_roundtrip() {
        let input = lipsum(4096);
        let input = input.as_bytes()[..4096].to_vec();

        let compressed = compress_palmdoc(&input);
        let decompressed = decompress_palmdoc(&compressed);
        assert_eq!(input, decompressed);
    }
}
