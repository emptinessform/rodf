//! rodf CLI 통합 테스트 — `rodf render in.odt out.{pdf,png}`.

use std::path::PathBuf;
use std::process::Command;

fn fixture_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../rodf-core/tests/fixtures/hello.odt"
    ))
}

fn out_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rodf-cli-tests");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn render_writes_pdf() {
    let out = out_path("hello.pdf");
    let _ = std::fs::remove_file(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_rodf"))
        .args(["render"])
        .arg(fixture_path())
        .arg(&out)
        .status()
        .expect("rodf should run");
    assert!(status.success());
    let bytes = std::fs::read(&out).expect("pdf should be written");
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn render_writes_png() {
    let out = out_path("hello.png");
    let _ = std::fs::remove_file(&out);
    let status = Command::new(env!("CARGO_BIN_EXE_rodf"))
        .args(["render"])
        .arg(fixture_path())
        .arg(&out)
        .status()
        .expect("rodf should run");
    assert!(status.success());
    let bytes = std::fs::read(&out).expect("png should be written");
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[test]
fn missing_input_fails_with_nonzero_exit() {
    let status = Command::new(env!("CARGO_BIN_EXE_rodf"))
        .args(["render", "no-such-file.odt"])
        .arg(out_path("never.pdf"))
        .status()
        .expect("rodf should run");
    assert!(!status.success());
}
