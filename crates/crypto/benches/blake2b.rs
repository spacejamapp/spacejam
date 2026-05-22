//! Microbench: BLAKE2b-256 across input sizes representative of the hot paths
//! (trie31 leaves/branches at 32–64B, header/preimage at 100–1KiB, work bundles
//! at 8–64KiB). Compares the in-tree blake2b_simd impl against the RustCrypto
//! `blake2` crate side by side.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn simd(input: &[u8]) -> [u8; 32] {
    spacejam_crypto::blake2b(input)
}

fn rustcrypto(input: &[u8]) -> [u8; 32] {
    use blake2::{digest::consts::U32, Blake2b, Digest};
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake2b");
    for &n in &[32usize, 64, 128, 512, 1024, 8192, 65536] {
        let input = vec![0xa5u8; n];
        group.throughput(Throughput::Bytes(n as u64));
        group.bench_with_input(BenchmarkId::new("simd", n), &input, |b, input| {
            b.iter(|| simd(black_box(input)));
        });
        group.bench_with_input(BenchmarkId::new("rustcrypto", n), &input, |b, input| {
            b.iter(|| rustcrypto(black_box(input)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
