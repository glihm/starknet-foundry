use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_transaction_hash, get_transaction_receipt,
};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::Declare;
use std::fs;
use test_case::test_case;

#[test_case("/v1/map", "1", "user7" ; "when cairo1 contract")]
#[test_case("/v2/map", "1", "user8" ; "when cairo2 contract")]
#[tokio::test]
async fn test_happy_case(contract_path: &str, salt: &str, account: &str) {
    let contract_path =
        duplicate_directory_with_salt(CONTRACTS_DIR.to_string() + contract_path, "put", salt);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        account,
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&contract_path)
        .args(args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));

    fs::remove_dir_all(contract_path).unwrap();
}

#[tokio::test]
async fn contract_already_declared() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "Map",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/v1/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: declare
        [..] is already declared.
        ...
    "#});
}

#[tokio::test]
async fn wrong_contract_name_passed() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "nonexistent",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/v1/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: declare
        error: Cannot find sierra artifact nonexistent in starknet_artifacts.json[..]
    "#});
}

#[test_case("/v1/build_fails", "../../../accounts/accounts.json" ; "when wrong cairo1 contract")]
#[test_case("/v2/build_fails", "../../../accounts/accounts.json" ; "when wrong cairo2 contract")]
#[test_case("/v1", "../../accounts/accounts.json" ; "when cairo 1 and Scarb.toml does not exist")]
#[test_case("/v2", "../../accounts/accounts.json" ; "when cairo 2 and Scarb.toml does not exist")]
fn scarb_build_fails(contract_path: &str, accounts_file_path: &str) {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file_path,
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + contract_path)
        .args(args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: declare
        error: Scarb build returned non-zero exit code: 1[..]
        ...
    "#});
}

#[test_case("/v1/map", "2", "user1" ; "when cairo1 contract")]
#[test_case("/v2/map", "2", "user2" ; "when cairo2 contract")]
fn test_too_low_max_fee(contract_path: &str, salt: &str, account: &str) {
    let contract_path =
        duplicate_directory_with_salt(CONTRACTS_DIR.to_string() + contract_path, "put", salt);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        account,
        "--wait",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&contract_path)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: declare
        error: Transaction has been rejected
    "#});

    fs::remove_dir_all(contract_path).unwrap();
}

#[test_case("/v2/no_sierra", "../../../accounts/accounts.json" ; "when there is no sierra artifact")]
#[test_case("/v2/no_casm", "../../../accounts/accounts.json" ; "when there is no casm artifact")]
fn scarb_no_artifacts(contract_path: &str, accounts_file_path: &str) {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file_path,
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + contract_path)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: declare
        [..]is set to 'true' under your [[target.starknet-contract]] field in Scarb.toml[..]
    "#});
}
