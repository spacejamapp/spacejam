//! Is authorized tests

use anyhow::Result;
use pvm::{
    Account, Invocation,
    score::{
        ServiceId,
        service::{RefineContext, ServiceAccount, WorkPackage},
    },
};
use std::{collections::BTreeMap, fs};

const AUTH_SERVICE: ServiceId = 500;

fn run_is_authorized<VM: Invocation>() -> Result<()> {
    let Ok(code) = fs::read("../../../res/services/nauth.jam") else {
        return Ok(());
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .without_time()
        .with_ansi(false)
        .init();

    let mut account = ServiceAccount::default();
    let hash = account.add_preimage(code, 0);

    let mut accounts = BTreeMap::new();
    accounts.insert(AUTH_SERVICE, account);

    tracing::debug!("auth code hash: 0x{}", hex::encode(hash));
    let package = WorkPackage {
        auth_code_host: AUTH_SERVICE,
        auth_code_hash: hash,
        context: RefineContext::default(),
        authorization: vec![],
        config: vec![],
        items: vec![],
    };

    let result = VM::is_authorized(&package, 0, &mut accounts, 0);
    tracing::debug!("result: {:?}", result);
    Ok(())
}

#[test]
fn test_interp_authorize() {
    run_is_authorized::<pvmi::Interpreter>().unwrap();
}

/* #[test]
fn test_comp_authorize() {
    run_is_authorized::<pvmc::Compiler>().unwrap();
} */
