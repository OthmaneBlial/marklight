use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_marklight"))
}
fn fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/markdown/gfm.md")
}

#[test]
fn file_plain_redirect_and_stdin_have_equal_output() {
    let file = cli()
        .arg(fixture())
        .args(["--plain", "--width", "60", "--no-pager"])
        .output()
        .unwrap();
    assert!(file.status.success());
    assert_eq!(
        String::from_utf8(file.stdout.clone()).unwrap(),
        include_str!("../../../fixtures/markdown/gfm.terminal.txt")
    );
    let default = cli()
        .arg(fixture())
        .args(["--width", "60"])
        .output()
        .unwrap();
    assert_eq!(file.stdout, default.stdout);
    let mut process = cli()
        .args(["-", "--plain", "--width", "60"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(&std::fs::read(fixture()).unwrap())
        .unwrap();
    let stdin = process.wait_with_output().unwrap();
    assert!(stdin.status.success());
    assert_eq!(file.stdout, stdin.stdout);
}

#[test]
fn errors_have_useful_exit_codes_without_panics() {
    let missing = cli().arg("/does/not/exist.md").output().unwrap();
    assert_eq!(missing.status.code(), Some(1));
    let message = String::from_utf8(missing.stderr).unwrap();
    assert!(message.starts_with("Marklight: unable to open"));
    assert!(!message.contains("panicked"));
    assert_eq!(
        cli().args(["--width", "1"]).output().unwrap().status.code(),
        Some(2)
    );
    assert_eq!(
        cli().args(["open", "-"]).output().unwrap().status.code(),
        Some(1)
    );
    assert!(cli().arg("--help").output().unwrap().status.success());
    assert!(cli().arg("--version").output().unwrap().status.success());
}

#[test]
fn directory_reads_readme_with_priority() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("README.md"), "# Readme").unwrap();
    std::fs::write(tmp.path().join("index.md"), "# Index").unwrap();
    let out = cli().arg(tmp.path()).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("Readme"));
}

#[cfg(unix)]
#[test]
fn gui_bridge_passes_canonical_document_as_one_argument() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let mock = tmp.path().join("desktop");
    std::fs::write(
        &mock,
        "#!/bin/sh\nprintf '%s' \"$1\" > \"$MARKLIGHT_TEST_OUT\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&mock, std::fs::Permissions::from_mode(0o700)).unwrap();
    for args in [vec!["open"], vec![]] {
        let record = tmp.path().join("argument.txt");
        let result = cli()
            .args(&args)
            .arg(fixture())
            .args(if args.is_empty() {
                vec!["--gui"]
            } else {
                vec![]
            })
            .env("MARKLIGHT_DESKTOP", &mock)
            .env("MARKLIGHT_TEST_OUT", &record)
            .output()
            .unwrap();
        assert!(result.status.success());
        for _ in 0..100 {
            if record.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            std::fs::read_to_string(&record).unwrap(),
            fixture().canonicalize().unwrap().to_string_lossy()
        );
        std::fs::remove_file(record).unwrap();
    }
}
