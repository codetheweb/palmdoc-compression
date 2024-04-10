use crate::{
    hashtable::HashTable,
    window::{Window, MAX_MATCH_LEN},
};
use thiserror::Error;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());

    let mut window = Window::new();
    let mut table = HashTable::new();

    let mut offset = 0;
    while offset < data.len() {
        let remainder = &data[offset..];
        if remainder.len() > 3 {
            let hash = table.hash(&remainder[..3]);
            table.insert(hash, window.position as u16);
            if let Some((distance, length)) = table.reference(hash, remainder, &window, offset) {
                // todo: this matches Calibre behavior where it doesn't encode length distance pairs that are close to the beginning or end of the data, but is this an actual PalmDoc limitation?
                if MAX_MATCH_LEN < offset && offset < data.len() - MAX_MATCH_LEN {
                    let m = distance as u16;
                    let code = 0x8000 + ((m << 3) & 0x3ff8) + ((length as u16) - 3);
                    out.extend(&code.to_be_bytes());

                    for _ in 0..length {
                        if offset + 3 < data.len() {
                            let hash = table.hash(&data[offset..offset + 3]);
                            table.insert(hash, window.position as u16);
                        }
                        window.push(data[offset]);
                        offset += 1;
                    }

                    continue;
                }
            }
        }

        // Single byte encoding or special cases handling
        let byte = data[offset];
        offset += 1;
        window.push(byte);

        if byte == b' ' && offset + 1 < data.len() && (0x40..0x80).contains(&data[offset]) {
            out.push(data[offset] ^ 0x80);

            if offset + 3 < data.len() {
                table.insert(table.hash(&data[offset..offset + 3]), offset as u16);
            }

            window.push(data[offset]);
            offset += 1;
            continue;
        }

        if byte == 0 || (byte > 8 && byte < 0x80) {
            out.push(byte);
        } else {
            let mut j = offset;
            let mut binseq = Vec::with_capacity(8);
            binseq.push(byte);

            while j < data.len() && binseq.len() < 8 {
                let ch = data[j];
                if ch == 0 || (ch > 8 && ch < 0x80) {
                    break;
                }

                binseq.push(ch);

                if j + 3 < data.len() {
                    table.insert(table.hash(&data[j..j + 3]), j as u16);
                }
                window.push(ch);

                j += 1;
            }

            out.extend(&(binseq.len() as u8).to_be_bytes());
            out.extend(&binseq);
            offset += binseq.len() - 1;
        }
    }

    out
}

#[derive(Error, Debug)]
pub enum DecompressError {
    #[error("offset to LZ77 bits is outside of the data")]
    OffsetOutsideData,
    #[error("LZ77 decompression offset is invalid")]
    InvalidOffset,
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, DecompressError> {
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
                return Err(DecompressError::OffsetOutsideData);
            }

            let mut lz77 = u16::from_be_bytes([data[offset - 2], data[offset - 1]]);

            // Leftmost two bits are ID bits and need to be dropped
            lz77 &= 0x3fff;

            // Length is rightmost 3 bits + 3
            let lz77length = ((lz77 & 0x0007) as usize) + 3;

            // Remaining 11 bits are offset
            let lz77offset = (lz77 >> 3) as usize;

            if lz77offset < 1 {
                return Err(DecompressError::InvalidOffset);
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

    Ok(uncompressed)
}

#[cfg(test)]
mod tests {
    use lipsum::lipsum;
    use pretty_assertions::assert_eq;

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
                hex::decode("61626373646661736466616263646173646f66617373").unwrap(),
            ),
        ];
    }

    #[test]
    fn test_compress_palmdoc() {
        for (input, expected) in get_calibre_testcases() {
            let compressed = compress(&input);
            assert_eq!(compressed, expected);
        }
    }

    #[test]
    fn test_decompress_palmdoc() {
        for (expected, compressed) in get_calibre_testcases() {
            let decompressed = decompress(&compressed).unwrap();
            assert_eq!(decompressed, expected);
        }
    }

    #[test]
    fn test_roundtrip() {
        let input = lipsum(4096);
        let input = input.as_bytes()[..4096].to_vec();

        let compressed = compress(&input);
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }
}
