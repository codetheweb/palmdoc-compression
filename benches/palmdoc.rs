use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lipsum::lipsum;
use palmdoc_compression::{compress, decompress};
use rand::{seq::SliceRandom, SeedableRng};

const CHUNK_SIZE: usize = 4096;

fn war_and_peace(c: &mut Criterion) {
    let text = std::fs::read_to_string("resources/war_and_peace.txt").unwrap();
    let text = text.as_bytes().to_vec();
    let chunks = text.chunks_exact(CHUNK_SIZE).collect::<Vec<_>>();

    let mut group = c.benchmark_group("war_and_peace");
    group.throughput(criterion::Throughput::Bytes(CHUNK_SIZE as u64));
    group.bench_function("decompress", |b| {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let chunk = chunks.choose(&mut rng).unwrap();
        let compressed = compress(&chunk);

        b.iter(|| {
            decompress(black_box(&compressed)).unwrap();
        })
    });
    group.bench_function("compress", |b| {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let chunk = chunks.choose(&mut rng).unwrap();
        b.iter(|| {
            compress(black_box(&chunk));
        })
    });
}

fn lorem_ipsum(c: &mut Criterion) {
    let lorem_ipsum = lipsum(CHUNK_SIZE);
    let lorem_ipsum = lorem_ipsum.as_bytes()[..CHUNK_SIZE].to_vec();

    let mut group = c.benchmark_group("lorem_ipsum");
    group.throughput(criterion::Throughput::Bytes(lorem_ipsum.len() as u64));
    group.bench_function("decompress", |b| {
        let compressed = compress(&lorem_ipsum);

        b.iter(|| {
            decompress(black_box(&compressed)).unwrap();
        })
    });
    group.bench_function("compress", |b| {
        b.iter(|| {
            compress(black_box(&lorem_ipsum));
        })
    });
}

criterion_group!(benches, war_and_peace, lorem_ipsum);
criterion_main!(benches);
