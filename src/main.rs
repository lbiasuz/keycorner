mod core;
mod devices;
mod hid;
mod keycodes;
mod layers;
mod permissions;
mod setup;
mod shortcuts;
mod stream;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn err<T>(message: &str) -> Result<T> {
    Err(message.into())
}
fn string(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| "Expected string".into())
}
fn home() -> Result<PathBuf> {
    Ok(std::env::var_os("HOME").ok_or("HOME is unset")?.into())
}
fn config_root() -> Result<PathBuf> {
    Ok(std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or(home()?.join(".config")))
}
fn emit(v: &Value) -> Result<()> {
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, v)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}
// Short-lived helpers drain stdout/stderr concurrently to avoid pipe deadlocks.
// The connected input loop does not call this or run a polling timer.
fn run(program: &str, args: &[&str]) -> Result<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("Missing stdout")?;
    let stderr = child.stderr.take().ok_or("Missing stderr")?;
    let reader = |mut input: std::process::ChildStdout| -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        input
            .by_ref()
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    };
    let out = std::thread::spawn(move || reader(stdout));
    let errors = std::thread::spawn(move || {
        let mut b = Vec::new();
        stderr.take(65537).read_to_end(&mut b).map(|_| b)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            let _ = child.wait();
            return err("Helper timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let bytes = out.join().map_err(|_| "Helper reader failed")??;
    let errors = errors.join().map_err(|_| "Helper reader failed")??;
    if bytes.len() > 4 * 1024 * 1024 {
        return err("Helper output too large");
    }
    if !status.success() {
        return err(&format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&errors)
        ));
    }
    Ok(String::from_utf8(bytes)?)
}
fn atomic(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let name = format!(
        ".keycorner-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    let temp = path.parent().ok_or("Missing parent")?.join(name);
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&temp)?;
        f.write_all(data)?;
        f.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    let _ = fs::remove_file(temp);
    result
}
fn theme() -> Value {
    if let Ok(home) = home() {
        for dir in [
            ".local/state/omarchy/current/theme",
            ".config/omarchy/current/theme",
        ] {
            if let Ok(f) = fs::File::open(home.join(dir).join("colors.toml")) {
                let mut data = String::new();
                if f.take(65537).read_to_string(&mut data).is_ok()
                    && data.len() <= 65536
                    && let Ok(v) = data.parse::<toml::Value>()
                    && let Ok(value) = serde_json::to_value(v)
                {
                    return value;
                }
            }
        }
    }
    json!({"background":"#1a1b26","foreground":"#c0caf5","accent":"#7aa2f7","muted":"#414868","lighter_background":"#24283b","dark_foreground":"#9aa5ce"})
}
fn dispatch(args: &[String]) -> Result<Value> {
    match args.first().map(String::as_str) {
        Some("theme") => Ok(json!({"theme":theme()})),
        Some("inspect") => stream::inspect(&args[1..]),
        Some("devices") => Ok(json!({"ok":true,"devices":devices::discover()?})),
        Some("config-init") => {
            setup::config_init()?;
            Ok(json!({"ok":true}))
        }
        Some("shortcuts") => shortcuts::command(&args[1..]),
        Some("permissions") => permissions::command(&args[1..]),
        Some("install") => {
            setup::install(&args[1..])?;
            Ok(json!({"ok":true}))
        }
        Some("bridge") => {
            stream::main(&args[1..])?;
            Ok(Value::Null)
        }
        Some("--version") => Ok(
            json!({"name":"keycorner","version":env!("CARGO_PKG_VERSION"),"implementation":"rust"}),
        ),
        _ => err(
            "Usage: keycorner bridge|theme|devices|inspect|config-init|shortcuts|permissions|install|--version",
        ),
    }
}
fn main() {
    // Never retain keyboard state in a core dump; applies to every entry point.
    unsafe {
        libc::setrlimit(
            libc::RLIMIT_CORE,
            &libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            },
        );
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(v) if !v.is_null() => {
            let _ = emit(&v);
        }
        Ok(_) => (),
        Err(e) => {
            let _ = emit(&json!({"ok":false,"error":e.to_string()}));
            std::process::exit(1)
        }
    }
}

#[cfg(test)]
mod tests;
