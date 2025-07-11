use spacejam_core::service::ServiceAccount;

#[test]
fn test_service_account() {
    let account = ServiceAccount::default();
    let encoded = codec::encode(&account).unwrap();
    assert_eq!(encoded, vec![0; 59]);
    let decoded = codec::decode::<ServiceAccount>(&encoded).unwrap();
    assert_eq!(account, decoded);
}
