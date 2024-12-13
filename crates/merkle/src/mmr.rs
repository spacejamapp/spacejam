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

        root = Some(crypto::keccak(&[next_peak, next_root].concat()));
    }

    peaks
}
