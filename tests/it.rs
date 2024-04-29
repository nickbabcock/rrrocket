use assert_cmd::cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_error_output() {
    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&[
            "-n",
            "-c",
            "--dry-run",
            "non-exist/assets/fuzz-string-too-long.replay",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Unable to parse replay non-exist/assets/fuzz-string-too-long.replay",
        ));
}

#[test]
fn test_error_output2() {
    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&[
            "-n",
            "-c",
            "--dry-run",
            "-m",
            "assets/fuzz-string-too-long.replay",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Unable to parse replay assets/fuzz-string-too-long.replay",
        ))
        .stderr(predicate::str::contains(
            "Crc mismatch. Expected 3765941959 but received 1825689991",
        ));
}

#[test]
fn test_file_in_stdout() {
    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n", "assets/replays/1ec9.replay"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#"{"header_size":1944,"header_crc":3561912561"#,
        ));
}

#[test]
fn test_stdin_stdout() {
    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n"])
        .pipe_stdin("assets/replays/1ec9.replay")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#"{"header_size":1944,"header_crc":3561912561"#,
        ));
}

#[test]
fn test_directory_in() {
    let dir = tempdir().unwrap();
    let options = fs_extra::dir::CopyOptions::new();
    let replays_path = dir.path().join("replays");
    let path = replays_path.to_str().unwrap().to_owned();
    fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();

    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n", "-m", &path])
        .assert()
        .success();

    assert!(replays_path.join("1ec9.replay.json").as_path().exists());
}

#[test]
fn test_directory_in_json_lines() {
    let dir = tempdir().unwrap();
    let options = fs_extra::dir::CopyOptions::new();
    let replays_path = dir.path().join("replays");
    let path = replays_path.to_str().unwrap().to_owned();
    fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();

    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n", "-j", "-m", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#"{"header_size":1944,"header_crc":3561912561"#,
        ))
        .stdout(predicate::str::contains("\n").count(1));
}

#[test]
fn test_directory_in_json_lines_nested() {
    let dir = tempdir().unwrap();
    let options = fs_extra::dir::CopyOptions::new();
    let replays_path = dir.path().join("replays");
    let path = replays_path.to_str().unwrap().to_owned();
    fs_extra::dir::copy("assets/replays", dir.path(), &options).unwrap();
    fs_extra::dir::copy("assets/replays", replays_path, &options).unwrap();

    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n", "-j", "-m", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"{"header_size":1944,"header_crc":3561912561"#).count(2))
        .stdout(predicate::str::contains("\n").count(2));
}

#[test]
fn test_zip() {
    Command::cargo_bin("rrrocket")
        .unwrap()
        .args(&["-n", "--dry-run", "assets/replays.zip"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"Parsed replays/1ec9.replay"#));
}
