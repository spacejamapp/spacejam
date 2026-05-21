//! Microbench: single ring-vrf verify vs batched verify, swept across batch sizes.

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use spacejam_crypto::{
    ring::RING_SIZE,
    vrf::{KeyPair, Verifier},
};

/// Generate `n` deterministic (message, signature) triples signed under distinct ring positions.
fn gen_items(n: usize) -> (Verifier, Vec<(Vec<u8>, [u8; 784])>) {
    assert!(n <= RING_SIZE, "batch size {n} exceeds RING_SIZE {RING_SIZE}");
    let ring: Vec<KeyPair> = (0..RING_SIZE).map(|i| KeyPair::from([i as u8; 32])).collect();
    let pks = ring
        .iter()
        .map(|k| k.public().unwrap())
        .collect::<Vec<_>>();
    let pkeys = ring.iter().map(|k| k.public).collect::<Vec<_>>();
    let verifier = Verifier::new(pkeys);

    let items: Vec<(Vec<u8>, [u8; 784])> = (0..n)
        .map(|i| {
            let msg = format!("benchmark ring-vrf message {i:08}").into_bytes();
            let sig = ring[i].ring_sign(pks.clone(), &msg, &[]).unwrap();
            (msg, sig)
        })
        .collect();

    (verifier, items)
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_vrf");
    group.sample_size(20);

    for &n in [1usize, 3, 8, 16, 32, 64].iter().filter(|&&n| n <= RING_SIZE) {
        let (verifier, items) = gen_items(n);
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("single", n), &items, |b, items| {
            b.iter(|| {
                for (m, s) in items {
                    verifier.ring_vrf_verify(black_box(m), &[], s).unwrap();
                }
            })
        });

        group.bench_with_input(BenchmarkId::new("batch", n), &items, |b, items| {
            b.iter(|| {
                verifier
                    .ring_vrf_verify_batch(
                        items.iter().map(|(m, s)| (m.as_slice(), [].as_slice(), s.as_slice())),
                    )
                    .unwrap()
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
