//! Tests for paritydb

use anyhow::Result;
use parity_db::{ColumnOptions, Db, Options};
use temp_dir::TempDir;

const COLUMN: u8 = 0;
const TEST_DATA: &[(&[u8], &[u8])] = &[
    (b"key", b"value"),
    (b"foo", b"bar"),
    (b"key1", b"value1"),
    (b"key2", b"value2"),
    (b"key3", b"value3"),
];

#[test]
fn test_seek_non_exist_key() -> Result<()> {
    let db = create_db()?;
    let mut iter = db.iter(COLUMN)?;
    assert!(iter.seek(b"non-exist-key").is_ok());
    assert_eq!(iter.next()?, None);
    Ok(())
}

#[test]
fn test_seek_exist_key() -> Result<()> {
    let db = create_db()?;
    let mut iter = db.iter(COLUMN)?;
    assert!(iter.seek(b"key").is_ok());
    assert_eq!(iter.next()?, Some((b"key".to_vec(), b"value".to_vec())));
    assert_eq!(iter.next()?, Some((b"key1".to_vec(), b"value1".to_vec())));
    assert_eq!(iter.next()?, Some((b"key2".to_vec(), b"value2".to_vec())));
    assert_eq!(iter.next()?, Some((b"key3".to_vec(), b"value3".to_vec())));
    assert_eq!(iter.next()?, None);
    Ok(())
}

#[test]
fn test_seek_partial_key_not_working() -> Result<()> {
    let db = create_db()?;
    let mut iter = db.iter(COLUMN)?;
    assert!(iter.seek(b"ey").is_ok());
    assert_eq!(iter.next()?, Some((b"foo".to_vec(), b"bar".to_vec())));
    assert_eq!(iter.next()?, Some((b"key".to_vec(), b"value".to_vec())));
    assert_eq!(iter.next()?, Some((b"key1".to_vec(), b"value1".to_vec())));
    assert_eq!(iter.next()?, Some((b"key2".to_vec(), b"value2".to_vec())));
    assert_eq!(iter.next()?, Some((b"key3".to_vec(), b"value3".to_vec())));
    assert_eq!(iter.next()?, None);
    Ok(())
}

fn create_db() -> Result<Db> {
    let path = TempDir::new()?;
    let options = Options {
        path: path.path().to_path_buf(),
        columns: vec![ColumnOptions {
            btree_index: true,
            ..Default::default()
        }],
        sync_wal: true,
        sync_data: true,
        stats: true,
        salt: None,
        compression_threshold: Default::default(),
    };

    let db = Db::open_or_create(&options)?;
    db.commit(
        TEST_DATA
            .iter()
            .map(|(k, v)| (COLUMN, *k, Some(v.to_vec())))
            .collect::<Vec<_>>(),
    )?;

    Ok(db)
}
