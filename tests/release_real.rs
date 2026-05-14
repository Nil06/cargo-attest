use assert_cmd::Command;
use predicates::str::contains;

#[test]
#[ignore = "hits GitHub and downloads a real release asset"]
fn ripgrep_release_with_sha256_sidecar_is_trusted() {
    let mut cmd = Command::cargo_bin("cargo-attest").unwrap();

    cmd.args([
        "attest",
        "release",
        "BurntSushi/ripgrep",
        "14.1.1",
        "--asset",
        "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz",
    ]);

    cmd.assert().success().stdout(contains("TRUSTED"));
}
