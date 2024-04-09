#![no_main]

use libfuzzer_sys::fuzz_target;
use palmdoc_compression::palmdoc::{compress_palmdoc, decompress_palmdoc};

fuzz_target!(|data: &[u8]| {
    let compressed = compress_palmdoc(data);
    let decompressed = decompress_palmdoc(&compressed);

    assert_eq!(&decompressed[..], data);
});
