use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
};
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "keycorner-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(p.join("config/hypr")).unwrap();
        fs::create_dir_all(p.join("bin")).unwrap();
        Self(p)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_keycorner"));
        c.env("HOME", &self.0)
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("PATH", format!("{}:/usr/bin", self.0.join("bin").display()));
        c
    }
    fn helper(&self, name: &str, body: &str) {
        let p = self.0.join("bin").join(name);
        fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o700)).unwrap();
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn config_creation_preserves_existing_invalid_file() {
    let s = Sandbox::new();
    assert!(
        s.command()
            .arg("config-init")
            .output()
            .unwrap()
            .status
            .success()
    );
    let path = s.0.join("config/omarchy/keycorner.json");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let fresh: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(fresh["backend"], "standard");
    assert!(fresh.get("deviceId").is_none());
    assert!(fresh.get("keyboards").is_none());
    fs::write(&path, "malformed but preserve").unwrap();
    assert!(
        s.command()
            .arg("config-init")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(path).unwrap(), "malformed but preserve");
}
#[test]
fn shortcut_failure_rolls_back_and_custom_lua_is_preserved() {
    let s = Sandbox::new();
    s.helper("hyprctl",r#"case "$1" in
-j) echo '[]';;
configerrors) if grep -q BAD "$XDG_CONFIG_HOME/hypr/keycorner-shortcuts.lua"; then echo 'invalid key'; else echo ok; fi;;
esac"#);
    let path = s.0.join("config/hypr/keycorner-shortcuts.lua");
    let original = include_str!("../config/shortcuts.lua");
    fs::write(&path, original).unwrap();
    let output = s
        .command()
        .args([
            "shortcuts",
            "apply",
            r#"{"toggle":"SUPER + BAD","toggleSettings":"","pause":""}"#,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    fs::write(&path, "-- custom\nprint('keep')\n").unwrap();
    assert!(
        !s.command()
            .args([
                "shortcuts",
                "apply",
                r#"{"toggle":"","toggleSettings":"","pause":""}"#
            ])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "-- custom\nprint('keep')\n"
    );
}
#[test]
fn installer_idempotence_and_validation_rollback() {
    let s = Sandbox::new();
    s.helper("omarchy", "exit 0");
    s.helper("hyprctl",r#"case "$1" in
-j) echo '[]';;
configerrors) if grep -q BAD "$XDG_CONFIG_HOME/hypr/keycorner.lua" 2>/dev/null; then echo invalid; else echo ok; fi;;
esac"#);
    let source = s.0.join("source");
    fs::create_dir_all(source.join("config")).unwrap();
    fs::write(source.join("config/hyprland.lua"), "BAD").unwrap();
    fs::write(
        source.join("config/shortcuts.lua"),
        include_str!("../config/shortcuts.lua"),
    )
    .unwrap();
    let main = s.0.join("config/hypr/hyprland.lua");
    fs::write(&main, "-- existing\n").unwrap();
    let invoke = || {
        s.command()
            .args([
                "install",
                "--setup-only",
                "--source",
                source.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    assert!(!invoke().status.success());
    assert_eq!(fs::read_to_string(&main).unwrap(), "-- existing\n");
    assert!(!s.0.join("config/hypr/keycorner.lua").exists());
    fs::write(source.join("config/hyprland.lua"), "-- good").unwrap();
    assert!(invoke().status.success());
    let after = fs::read(&main).unwrap();
    fs::write(s.0.join("config/hypr/keycorner.lua"), "-- custom rule").unwrap();
    assert!(invoke().status.success());
    assert_eq!(fs::read(main).unwrap(), after);
    assert_eq!(
        fs::read_to_string(s.0.join("config/hypr/keycorner.lua")).unwrap(),
        "-- custom rule"
    );
}
#[test]
fn parent_eof_stops_reconnect_wait() {
    let s = Sandbox::new();
    let mut child = s
        .command()
        .args(["bridge", "--device", "invalid"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(line.contains("Select a keyboard"));
    drop(child.stdin.take());
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(3) {
            child.kill().unwrap();
            panic!("Bridge did not stop on EOF")
        };
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn marketplace_bootstrap_without_cargo_is_repeatable_and_checks_architecture() {
    let s = Sandbox::new();
    s.helper("omarchy", "exit 0");
    s.helper("cargo", "exit 99");
    s.helper(
        "hyprctl",
        r#"echo "$1" >> "$HOME/hypr-calls"
case "$1" in
-j) echo '[]';;
configerrors) echo ok;;
esac"#,
    );
    let source = s.0.join("plugin");
    fs::create_dir_all(source.join("bin")).unwrap();
    fs::create_dir(source.join("config")).unwrap();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "keycorner.sh",
        "config/hyprland.lua",
        "config/shortcuts.lua",
    ] {
        fs::copy(repo.join(file), source.join(file)).unwrap();
    }
    fs::write(s.0.join("config/hypr/hyprland.lua"), "-- user config\n").unwrap();
    let invoke = || {
        let mut command = s.command();
        // Use the same isolated environment, but run the real bootstrap script.
        let envs: Vec<_> = command
            .get_envs()
            .map(|(k, v)| (k.to_owned(), v.unwrap().to_owned()))
            .collect();
        command = Command::new("bash");
        command
            .envs(envs)
            .arg(source.join("keycorner.sh"))
            .arg("bootstrap")
            .output()
            .unwrap()
    };
    let missing = invoke();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stdout).contains("executable missing"));
    fs::copy(
        env!("CARGO_BIN_EXE_keycorner"),
        source.join("bin/keycorner"),
    )
    .unwrap();
    let first = invoke();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    let config = s.0.join("config/omarchy/keycorner.json");
    fs::write(&config, "{\"version\":1,\"shown\":false,\"paused\":true}\n").unwrap();
    fs::write(s.0.join("hypr-calls"), "").unwrap();
    assert!(invoke().status.success());
    assert_eq!(fs::read_to_string(s.0.join("hypr-calls")).unwrap(), "");
    assert!(
        fs::read_to_string(config)
            .unwrap()
            .contains("\"paused\":true")
    );
    s.helper("uname", "echo aarch64");
    let unsupported = invoke();
    assert!(!unsupported.status.success());
    assert!(String::from_utf8_lossy(&unsupported.stdout).contains("x86_64"));
}
