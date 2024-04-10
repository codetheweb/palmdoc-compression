# 🖐️ palmdoc-compression

[![docs.rs](https://img.shields.io/docsrs/palmdoc-compression)](https://docs.rs/palmdoc-compression/latest/palmdoc_compression/)

This is a fast, safe, and correct implementation of PalmDoc-flavored LZ77 compression (primarily used by Amazon ebook formats). Compression is **300-400x** faster than Calibre's implementation with a comparable compression ratio.

This crate also includes Calibre's version for comparison and usage if desired, gated behind the `calibre` feature.

## Usage

```rust
use palmdoc_compression::{compress, decompress};

let data = b"hello world";

let compressed = compress(data);
let decompressed = decompress(&compressed).unwrap();

assert_eq!(data, decompressed);
```

## ⚡ Benchmarks

MOBI/AZW files are split into 4KB chunks, so benchmarks here also use 4KB chunks. Benchmarks were run on a M1 Max.

For a 4KB chunk of lorem ipsum text:

|                     | Decompression | Compression |
|---------------------|---------------|-------------|
| Calibre             | 922 MiB/s     | 252 KiB/s   |
| palmdoc-compression | 797 MiB/s     | 109 MiB/s   |


For a random 4KB chunk of War and Peace from Project Gutenberg:

|                     | Decompression | Compression |
|---------------------|---------------|-------------|
| Calibre             | 1011 MiB/s    | 336 KiB/s   |
| palmdoc-compression | 876 MiB/s     | 103 MiB/s   |

(Reproduce with `cargo bench --features calibre`.)

## Compression ratio

Ratios calculated by compressing War and Peace from Project Gutenberg in 4KB chunks.

|                     | ratio, ⬇️ is better      |
|---------------------|-------------------------|
| calibre             | 0.56% (theoretical max) |
| palmdoc-compression | 0.57%                   |

(Reproduce with `cargo run --example ratios --release --features calibre`.)

## Credits

- [LPeter1997](https://github.com/LPeter1997) for a clear and understandable Rust LZ77 implementation with [hash chains](https://gist.github.com/LPeter1997/1c88e7540d03552cacd875eb82caad8d)
- Calibre for a reference implementation with tests
