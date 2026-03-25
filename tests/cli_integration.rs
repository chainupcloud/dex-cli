use assert_cmd::Command;
use predicates::prelude::*;

fn dex() -> Command {
    Command::cargo_bin("dex").unwrap()
}

#[test]
fn test_help() {
    dex()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("DEX CLI"));
}

#[test]
fn test_version() {
    dex()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("dex"));
}

#[test]
fn test_market_help() {
    dex()
        .args(["market", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("book"))
        .stdout(predicate::str::contains("trades"));
}

#[test]
fn test_order_help() {
    dex()
        .args(["order", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("place"))
        .stdout(predicate::str::contains("cancel"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_wallet_help() {
    dex()
        .args(["wallet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("import"))
        .stdout(predicate::str::contains("address"));
}

#[test]
fn test_admin_help() {
    dex()
        .args(["admin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("oracle-update"));
}

#[test]
fn test_watch_help() {
    dex()
        .args(["watch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trades"))
        .stdout(predicate::str::contains("book"))
        .stdout(predicate::str::contains("mids"));
}

#[test]
fn test_status_help() {
    dex()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api"))
        .stdout(predicate::str::contains("gateway"));
}

#[test]
fn test_json_output_flag() {
    dex()
        .args(["-o", "json", "market", "--help"])
        .assert()
        .success();
}

/// These tests override XDG config dir to ensure no wallet is found.
/// On macOS, dirs::config_dir() uses ~/Library/Application Support,
/// so we point HOME to a temp dir to isolate from real config.
#[test]
fn test_order_requires_identity() {
    dex()
        .env("HOME", "/tmp/dex-cli-test-nonexistent")
        .env_remove("DEX_PRIVATE_KEY")
        .env_remove("DEX_SENDER_INDEX")
        .args(["order", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No wallet configured"));
}

#[test]
fn test_account_requires_identity() {
    dex()
        .env("HOME", "/tmp/dex-cli-test-nonexistent")
        .env_remove("DEX_PRIVATE_KEY")
        .env_remove("DEX_SENDER_INDEX")
        .args(["account", "info"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No wallet configured"));
}

#[test]
fn test_agent_help() {
    dex()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("approve"))
        .stdout(predicate::str::contains("revoke"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"));
}

#[test]
fn test_agent_approve_help() {
    dex()
        .args(["agent", "approve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent-address"))
        .stdout(predicate::str::contains("valid-until"));
}

#[test]
fn test_agent_show_no_key() {
    dex()
        .env("HOME", "/tmp/dex-cli-test-nonexistent")
        .env_remove("DEX_PRIVATE_KEY")
        .env_remove("DEX_SENDER_INDEX")
        .args(["agent", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No agent key configured"));
}
