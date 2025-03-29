//! MMR utils.

const MMR_PEAK: [u8; 4] = *b"peak";

/// Append a root to the peaks of the MMR.
pub fn append(
    mut peaks: Vec<Option<[u8; 32]>>,
    accumulate_root: [u8; 32],
) -> Vec<Option<[u8; 32]>> {
    let mut root = Some(accumulate_root);
    let peaks_len = peaks.len();
    for n in 0..=peaks_len {
        if n >= peaks_len {
            peaks.push(root.take());
            continue;
        }

        if peaks[n].is_none() {
            peaks[n] = root.take();
            continue;
        }

        let Some(next_root) = root.take() else {
            break;
        };

        let Some(next_peak) = peaks[n].take() else {
            break;
        };

        root = Some(crate::keccak(&[next_peak, next_root].concat()));
    }

    peaks
}

/// Calculate the root of the MMR from the peaks.
pub fn root(peaks: &[Option<[u8; 32]>]) -> Option<[u8; 32]> {
    let non_empty_peaks: Vec<[u8; 32]> = peaks.iter().filter_map(|p| *p).collect();

    if non_empty_peaks.is_empty() {
        return None;
    }

    if non_empty_peaks.len() == 1 {
        return Some(non_empty_peaks[0]);
    }

    // Calculate super-peak
    let mut current = non_empty_peaks[0];
    for peak in non_empty_peaks.iter().skip(1) {
        let mut to_hash = vec![];
        to_hash.extend_from_slice(&MMR_PEAK);
        to_hash.extend_from_slice(&current);
        to_hash.extend_from_slice(peak);

        current = crate::keccak(&to_hash);
    }

    Some(current)
}

#[cfg(test)]
fn to_bytes(hex: &str) -> [u8; 32] {
    hex::decode(hex)
        .expect("Failed to decode hex")
        .try_into()
        .expect("Failed to convert to [u8; 32]")
}

#[test]
fn test_verify_beefy_root_valid() {
    // From reports_with_dependencies-1.json
    let peaks = vec![
        Some(to_bytes(
            "4c31a1024d553c6f5eb90a26f9c53507d6d58b7be1197c0f86054b084353de5f",
        )),
        None,
        Some(to_bytes(
            "7f64e54f8be039cea06582eb38e7f36f924c1f59a0f3043b4df6f140cccd6ddf",
        )),
        Some(to_bytes(
            "d7cc7a7751048dbe8d0232b5d0273acd874e56c19e41a2e09b590ca00e59908d",
        )),
    ];

    let beefy_root = to_bytes("f5df0c11416d43c55b43e096572d450b7780ed0fd7b540f26c8ded8e0d41e183");
    assert_eq!(root(&peaks), Some(beefy_root));
}

#[test]
fn test_verify_beefy_root_invalid() {
    // From bad_beefy_mmr-1.json
    let peaks = vec![
        Some(to_bytes(
            "4c31a1024d553c6f5eb90a26f9c53507d6d58b7be1197c0f86054b084353de5f",
        )),
        None,
        Some(to_bytes(
            "7f64e54f8be039cea06582eb38e7f36f924c1f59a0f3043b4df6f140cccd6ddf",
        )),
        Some(to_bytes(
            "d7cc7a7751048dbe8d0232b5d0273acd874e56c19e41a2e09b590ca00e59908d",
        )),
    ];

    let invalid_beefy_root =
        to_bytes("1248fd314e6ca467f0305f3494a66c75c37aa084512c6066ee211d49bb1f39bc");
    assert_ne!(root(&peaks), Some(invalid_beefy_root));
}
