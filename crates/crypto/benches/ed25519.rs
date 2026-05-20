//! Microbench: single-verify vs batch-verify, swept across batch sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use spacejam_crypto::ed25519::{batch_verify, verify, KeyPair};

/// Generate `n` deterministic (msg, sig, key) triples.
fn gen_items(n: usize) -> Vec<(Vec<u8>, [u8; 64], [u8; 32])> {
    (0..n)
        .map(|i| {
            let mut seed = [0u8; 32];
            seed[..8].copy_from_slice(&(i as u64).to_le_bytes());
            let kp = KeyPair::from(seed);
            let msg = format!("benchmark message {i:08}").into_bytes();
            let sig = kp.signing.sign(&msg).to_bytes();
            (msg, sig, kp.public())
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ed25519");
    // Sweep covers: well-below-chunk (1, 8), at-threshold (32), above (64, 128, 512).
    for &n in &[1usize, 8, 32, 64, 128, 512] {
        let items = gen_items(n);
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("single", n), &items, |b, items| {
            b.iter(|| {
                for (m, s, k) in items {
                    verify(black_box(m), *s, *k).unwrap();
                }
            })
        });

        let refs: Vec<_> = items
            .iter()
            .map(|(m, s, k)| (m.as_slice(), *s, *k))
            .collect();
        group.bench_with_input(BenchmarkId::new("batch", n), &refs, |b, refs| {
            b.iter(|| batch_verify(black_box(refs)).unwrap())
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
