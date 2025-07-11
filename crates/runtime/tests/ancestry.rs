/* use score::block::Head;

#[test]
fn test_advance() {
    let mut ancestry = Ancestry {
        best: Head {
            slot: 42,
            hash: [0; 32],
        },
        ancestors: (1..42).map(|i| [i; 32]).rev().collect(),
        finalized: Head {
            slot: 0,
            hash: [0; 32],
        },
    };

    for i in 1..42 {
        ancestry
            .advance(&Head {
                slot: i,
                hash: [i as u8; 32],
            })
            .expect("failed to advance");

        assert!(!ancestry.ancestors.contains(&[i as u8; 32]));
        assert_eq!(ancestry.best.slot, 42);
        assert_eq!(ancestry.finalized.slot, i);
        assert_eq!(ancestry.finalized.hash, [i as u8; 32]);
        assert_eq!(ancestry.ancestors.len(), 41 - i as usize);
        assert_eq!(
            ancestry.ancestors.last(),
            (i != 41).then_some(&[1 + i as u8; 32])
        );
    }
}
 */
