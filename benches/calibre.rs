use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lipsum::lipsum;
use palmdoc_compression::calibre::{compress, decompress};

fn random_worst_case(c: &mut Criterion) {
    let input = (0..4096).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

    let mut group = c.benchmark_group("calibre random");
    group.throughput(criterion::Throughput::Bytes(input.len() as u64));
    group.bench_function("decompress", |b| {
        let compressed = compress(&input);

        b.iter(|| {
            decompress(black_box(&compressed));
        })
    });
    group.bench_function("compress", |b| {
        b.iter(|| {
            compress(black_box(&input));
        })
    });
}

fn lorem_ipsum(c: &mut Criterion) {
    let lorem_ipsum = lipsum(4096);
    let lorem_ipsum = lorem_ipsum.as_bytes()[..4096].to_vec();

    let mut group = c.benchmark_group("calibre lorem ipsum");
    group.throughput(criterion::Throughput::Bytes(lorem_ipsum.len() as u64));
    group.bench_function("decompress", |b| {
        let compressed = compress(&lorem_ipsum);

        b.iter(|| {
            decompress(black_box(&compressed));
        })
    });
    group.bench_function("compress", |b| {
        b.iter(|| {
            compress(black_box(&lorem_ipsum));
        })
    });
}

criterion_group!(benches, random_worst_case, lorem_ipsum);
criterion_main!(benches);
