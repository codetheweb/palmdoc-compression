#![no_main]

use libfuzzer_sys::fuzz_target;
use palmdoc_compression::{compress, decompress};

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }

    let compressed = compress(data);
    let decompressed = decompress(&compressed).unwrap();

    assert_eq!(&decompressed[..], data);
});
