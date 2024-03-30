use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lipsum::lipsum;
use palmdoc_compression::palmdoc::{compress_palmdoc, decompress_palmdoc};

fn random_worst_case(c: &mut Criterion) {
    let input = (0..4096).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("palmdoc random");
    group.throughput(criterion::Throughput::Bytes(input.len() as u64));
    group.bench_function("decompress", |b| {
        let compressed = compress_palmdoc(&input);

        b.iter(|| {
            decompress_palmdoc(black_box(&compressed));
        })
    });
    group.bench_function("compress", |b| {
        b.iter(|| {
            compress_palmdoc(black_box(&input));
        })
    });
}

fn lorem_ipsum(c: &mut Criterion) {
    let lorem_ipsum = lipsum(4096);
    let lorem_ipsum = lorem_ipsum.as_bytes();

    let mut group = c.benchmark_group("palmdoc lorem ipsum");
    group.throughput(criterion::Throughput::Bytes(lorem_ipsum.len() as u64));
    group.bench_function("decompress", |b| {
        let compressed = compress_palmdoc(&lorem_ipsum);

        b.iter(|| {
            decompress_palmdoc(black_box(&compressed));
        })
    });
    group.bench_function("compress", |b| {
        b.iter(|| {
            compress_palmdoc(black_box(&lorem_ipsum));
        })
    });
}

criterion_group!(benches, random_worst_case, lorem_ipsum);
criterion_main!(benches);
