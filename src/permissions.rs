use crate::{Result, devices, err, run, string};
use serde_json::{Value, json};
use std::{fs, io::Write, os::unix::fs::OpenOptionsExt, path::PathBuf};
fn literal(v: &Value) -> Result<&str> {
    let s = string(v)?;
    if s.is_empty()
        || s.chars().count() > 256
        || s.chars()
            .any(|c| c < ' ' || c == '\u{7f}' || "\"\\*?[]$%".contains(c))
    {
        return err("Device identity contains unsupported rule characters");
    };
    Ok(s)
}
pub fn rule(d: &Value, backend: &str) -> Result<String> {
    if !matches!(backend, "standard" | "vial") {
        return err("Invalid backend");
    };
    if !devices::valid_id(string(&d["id"])?) {
        return err("Invalid device identifier");
    };
    let m = &d["match"];
    let mut selectors = Vec::new();
    if let Some(port) = m.get("usb_port") {
        let p = string(port)?;
        let (left, right) = p.split_once('-').ok_or("Invalid USB port")?;
        if left.is_empty()
            || right.is_empty()
            || !left.bytes().all(|b| b.is_ascii_digit())
            || !right.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        {
            return err("Invalid USB port");
        };
        for k in ["vendor", "product"] {
            let s = string(&m[k])?;
            if s.len() != 4 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
                return err("Invalid USB IDs");
            }
        }
        selectors.extend([
            format!("KERNELS==\"{p}\""),
            format!("ATTRS{{idVendor}}==\"{}\"", string(&m["vendor"])?),
            format!("ATTRS{{idProduct}}==\"{}\"", string(&m["product"])?),
        ]);
        if m["serial"].as_str().is_some_and(|s| !s.is_empty()) {
            selectors.push(format!("ATTRS{{serial}}==\"{}\"", literal(&m["serial"])?));
        }
    } else {
        if backend == "vial" {
            return err("Vial access requires a supported USB keyboard");
        };
        for k in ["name", "phys"] {
            selectors.push(format!("ATTRS{{{k}}}==\"{}\"", literal(&m[k])?));
        }
        if m["uniq"].as_str().is_some_and(|s| !s.is_empty()) {
            selectors.push(format!("ATTRS{{uniq}}==\"{}\"", literal(&m["uniq"])?));
        }
    }
    let mut parts = if backend == "vial" {
        vec![
            "SUBSYSTEM==\"hidraw\"".to_owned(),
            "KERNEL==\"hidraw*\"".into(),
        ]
    } else {
        vec![
            "SUBSYSTEM==\"input\"".into(),
            "KERNEL==\"event*\"".into(),
            "ENV{ID_INPUT_KEYBOARD}==\"1\"".into(),
        ]
    };
    parts.extend(selectors);
    parts.push("TAG+=\"uaccess\"".into());
    Ok(format!(
        "# Keycorner: selected keyboard only; active local session access.\n{}\n",
        parts.join(", ")
    ))
}
fn destination(d: &Value, backend: &str) -> Result<PathBuf> {
    rule(d, backend)?;
    Ok(PathBuf::from(format!(
        "/etc/udev/rules.d/69-keycorner-{}-{backend}.rules",
        string(&d["id"])?
    )))
}
fn root_install(d: &Value, backend: &str) -> Result<PathBuf> {
    if unsafe { libc::geteuid() } != 0 {
        return err("Administrator authentication required");
    };
    let content = rule(d, backend)?;
    let dest = destination(d, backend)?;
    let nodes = d["nodes"]
        .as_array()
        .filter(|v| !v.is_empty())
        .ok_or("Invalid device nodes")?;
    let prefix = if backend == "standard" {
        "/dev/input/event"
    } else {
        "/dev/hidraw"
    };
    for node in nodes {
        let suffix = string(node)?
            .strip_prefix(prefix)
            .ok_or("Invalid device nodes")?;
        if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
            return err("Invalid device nodes");
        }
    }
    let temp = dest.parent().unwrap().join(format!(
        ".keycorner-{}-{}.rules",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let result = (|| -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o644)
            .open(&temp)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
        run(
            "/usr/bin/udevadm",
            &["verify", temp.to_str().ok_or("Invalid path")?],
        )?;
        fs::rename(&temp, &dest)?;
        Ok(())
    })();
    let _ = fs::remove_file(temp);
    result?;
    run("/usr/bin/udevadm", &["control", "--reload-rules"])?;
    for node in nodes {
        run(
            "/usr/bin/udevadm",
            &["trigger", "--settle", "--action=change", string(node)?],
        )?;
    }
    Ok(dest)
}
pub fn command(args: &[String]) -> Result<Value> {
    if args.len() == 2 && args[0] == "--root-install" {
        let req: Value = serde_json::from_str(&args[1])?;
        return Ok(
            json!({"ok":true,"path":root_install(&req["device"],string(&req["backend"])?)?}),
        );
    };
    if args.len() != 3 {
        return err("Invalid permissions command");
    };
    let action = &args[0];
    let backend = &args[2];
    let mut d = devices::select(&args[1])?;
    d["nodes"] = d[if backend == "standard" {
        "events"
    } else {
        "hidraw"
    }]
    .clone();
    if backend == "vial" && d["vial"] != true {
        return err("No compatible raw HID interface; choose Standard mode");
    }
    let content = rule(&d, backend)?;
    let dest = destination(&d, backend)?;
    if action == "preview" {
        return Ok(json!({"ok":true,"rule":content,"path":dest,"device":d["name"]}));
    };
    if action != "install" {
        return err("Unknown permission action");
    };
    let request =
        json!({"device":{"id":d["id"],"match":d["match"],"nodes":d["nodes"]},"backend":backend})
            .to_string();
    // Authentication is deliberately interactive, without the helper timeout.
    let status = std::process::Command::new("/usr/bin/pkexec")
        .arg(std::env::current_exe()?)
        .args(["permissions", "--root-install", &request])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        return err(
            "Access setup cancelled or failed. Reconnect the keyboard if a rule was installed.",
        );
    };
    Ok(
        json!({"ok":true,"message":"Access enabled. Reconnect the keyboard if access is still unavailable.","path":dest}),
    )
}
