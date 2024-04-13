#![no_main]

use libfuzzer_sys::fuzz_target;
use palmdoc_compression::decompress;

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }

    decompress(&data).unwrap_or_default();
});
