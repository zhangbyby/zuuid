use std::io::BufRead;
use std::process::{Command, Output, Stdio};
use uuid::Uuid;

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zuuid"));
    command
        .env("LC_ALL", "C")
        .env("LC_MESSAGES", "C")
        .env("LANG", "C");
    command
}

fn run(args: &[&str]) -> Output {
    command().args(args).output().unwrap()
}

#[test]
fn test_first_format_flag_wins_without_scanning_option_names() {
    for (args, expected_length) in [
        (vec!["-fs"], 36),
        (vec!["-sf"], 32),
        (vec!["--uuid-version", "7", "-fs"], 36),
        (vec!["--uuid-version=7", "--full", "--simple"], 36),
        (vec!["--uuid-version", "7", "--simple", "--full"], 32),
        (vec!["-V7", "-FuS"], 36),
        (vec!["-v7", "-Suf"], 32),
    ] {
        let output = run(&args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(text.trim().len(), expected_length, "{args:?}");
        assert!(Uuid::parse_str(text.trim()).is_ok());
    }
}

#[test]
fn test_uuid_version_option_alone_does_not_create_format_conflict() {
    for args in [
        vec!["--uuid-version", "7", "-f"],
        vec!["--uuid-version=4", "--full"],
    ] {
        let output = run(&args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty(), "{args:?}: {output:?}");
    }
}

#[test]
fn test_program_version_preserves_uuid_version_short_option() {
    let output = run(&["--version"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        concat!("zuuid ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
    assert!(output.stderr.is_empty());
    let output = run(&["-V7"]);
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(Uuid::parse_str(text.trim()).unwrap().get_version_num(), 7);
}

#[test]
fn test_locale_uses_first_nonempty_value_in_standard_priority_order() {
    for (all, messages, lang, chinese) in [
        ("C", "zh_CN.UTF-8", "zh_CN.UTF-8", false),
        ("", "en_US.UTF-8", "zh_CN.UTF-8", false),
        ("", "", "zh_CN.UTF-8", true),
        ("zh_CN.UTF-8", "C", "C", true),
        ("", "", "", false),
    ] {
        let output = command()
            .args(["-V", "5"])
            .env("LC_ALL", all)
            .env("LC_MESSAGES", messages)
            .env("LANG", lang)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(if chinese {
                "无效的 UUID 版本"
            } else {
                "Invalid UUID version"
            }),
            "{all:?}/{messages:?}/{lang:?}: {error}"
        );
    }
}

#[test]
fn test_batch_count_case_and_uuid_version() {
    for version in ["4", "7"] {
        let output = run(&["-n", "3", "-U", "-s", "-V", version]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in lines {
            assert_eq!(line.len(), 32);
            assert!(
                line.bytes()
                    .all(|ch| ch.is_ascii_digit() || (b'A'..=b'F').contains(&ch))
            );
            let id = Uuid::parse_str(line).unwrap();
            assert_eq!(id.get_version_num().to_string(), version);
            assert_eq!(id.get_variant(), uuid::Variant::RFC4122);
        }
    }
}

#[test]
fn test_zero_count_produces_no_output() {
    let output = run(&["-n", "0"]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn test_closing_output_pipe_exits_without_panicking() {
    let mut child = command()
        .args(["-n", "100000"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    reader.read_line(&mut first).unwrap();
    assert!(Uuid::parse_str(first.trim()).is_ok());
    drop(reader);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[cfg(unix)]
#[test]
fn test_help_accepts_a_non_utf8_program_name() {
    use std::ffi::OsStr;
    use std::os::unix::{ffi::OsStrExt, process::CommandExt};
    let output = command()
        .arg0(OsStr::from_bytes(b"\xffzuuid"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Generate UUID"));
}
