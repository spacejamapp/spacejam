use score::{
    block::{Head, Header},
    TimeSlot,
};
use spacejam_runtime::{
    storage::{MemoryDb, SyncStorage},
    Grandpa,
};
use std::sync::Arc;

#[test]
fn test_select_best_head() {
    let db = MemoryDb::default();
    let ancestry = Arc::new(db);
    let mut grandpa = Grandpa::new(ancestry);
    let mut parent = Header {
        slot: 0,
        parent: [0; 32],
        ..Default::default()
    };

    grandpa.handshake.head = Head {
        slot: 0,
        hash: parent.hash().unwrap(),
    };
    grandpa.ancestry.set_header(&parent).unwrap();
    for i in 1..20u8 {
        let header = Header {
            slot: i as TimeSlot,
            parent: parent.hash().unwrap(),
            ..Default::default()
        };
        let hash = header.hash().unwrap();
        grandpa.add_leaf(header.clone()).unwrap();
        let best = grandpa.select_best_head();

        if i > 6 {
            let best = best.unwrap();
            assert_eq!(
                hex::encode(&best.best.hash.as_ref()),
                hex::encode(&hash.as_ref())
            );
        } else {
            assert!(best.is_none());
        }

        parent = header;
    }
}
