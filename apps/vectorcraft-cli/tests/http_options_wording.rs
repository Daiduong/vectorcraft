//! Regression: preserve the v0.14 user-facing local HTTP option labels and help text.
//! These strings are part of our private VectorCraft's CLI compatibility contract.
#![cfg(feature = "http")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_vectorcraft-cli");

#[test]
fn http_help_keeps_original_option_wording() {
    let output = Command::new(BIN).arg("--help").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let help = String::from_utf8(output.stdout).unwrap();
    const ORIGINAL: &str = "Experimental local HTTP:\n  vectorcraft-cli mcp --http 127.0.0.1:33981 --http-token-file PATH --connect 127.0.0.1:33982\n  Requires a private 64-hex-character bearer token file. No public binding or headless fallback.";
    assert!(help.contains(ORIGINAL), "original HTTP CLI help was changed:\n{help}");
}

#[test]
fn http_option_errors_keep_original_diagnostics() {
    for (args, message) in [
        (vec!["mcp", "--http"], "--http needs a loopback address"),
        (vec!["mcp", "--http-token-file"], "--http-token-file needs a path"),
        (vec!["mcp", "--http", "127.0.0.1:33981"], "--http requires --http-token-file"),
        (vec!["mcp", "--http-token-file", "unused"], "--http-token-file requires --http"),
    ] {
        let result = Command::new(BIN).args(&args).output().unwrap();
        assert!(!result.status.success(), "{args:?} unexpectedly succeeded");
        let err = String::from_utf8_lossy(&result.stderr);
        assert!(err.contains(message), "{args:?} changed expected diagnostic ({message}): {err}");
    }
}
