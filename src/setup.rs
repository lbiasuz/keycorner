use crate::{Result, atomic, err, run, shortcuts};
use std::{fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};
const FILES: &str = include_str!("../packaging/runtime-files.txt");
pub fn config_init() -> Result<()> {
    let path = crate::config_root()?.join("omarchy/keycorner.json");
    fs::create_dir_all(path.parent().unwrap())?;
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(mut f) => {
            f.write_all(b"{\"version\":1,\"backend\":\"standard\"}\n")?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e.into()),
    }
}
fn setup_include(
    source: &Path,
    hypr: &Path,
    main_name: &str,
    target_name: &str,
    template: &str,
    include: &str,
    is_shortcuts: bool,
) -> Result<()> {
    let main = hypr.join(main_name);
    let dest = hypr.join(target_name);
    let original = match fs::read(&main) {
        Ok(data) => data,
        Err(e) if is_shortcuts && e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e.into()),
    };
    let old = fs::read(&dest).ok();
    let included = std::str::from_utf8(&original)?.contains(include);
    if is_shortcuts && included {
        return Ok(());
    };
    if is_shortcuts {
        if old.is_some() {
            return err("Unmanaged keycorner-shortcuts.lua exists; include it manually");
        };
        let binds: serde_json::Value = serde_json::from_str(&run("hyprctl", &["-j", "binds"])?)?;
        let defaults = serde_json::json!({"toggle":"SUPER + CTRL + SHIFT + K","toggleSettings":"SUPER + CTRL + SHIFT + O","pause":"SUPER + CTRL + SHIFT + P"});
        shortcuts::conflicts(&defaults, &serde_json::json!({}), &binds)?;
    }
    let updated = if included {
        original.clone()
    } else {
        [
            original.as_slice(),
            format!("\n-- Keycorner.\n{include}\n").as_bytes(),
        ]
        .concat()
    };
    if original == updated && old.is_some() {
        return Ok(());
    }
    if original != updated {
        atomic(
            &main.with_extension(format!(
                "lua.keycorner-backup-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
            )),
            &original,
            0o600,
        )?;
    }
    let result = (|| -> Result<()> {
        if old.is_none() {
            atomic(&dest, &fs::read(source.join(template))?, 0o600)?;
        }
        if original != updated {
            atomic(&main, &updated, 0o600)?;
        }
        shortcuts::validate()
    })();
    if let Err(e) = result {
        atomic(&main, &original, 0o600)?;
        if let Some(bytes) = old {
            atomic(&dest, &bytes, 0o600)?
        } else {
            let _ = fs::remove_file(dest);
        }
        shortcuts::validate()?;
        return Err(e);
    }
    Ok(())
}
pub fn install(args: &[String]) -> Result<()> {
    let mut source = std::env::current_dir()?;
    let mut setup_only = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--source" => source = iter.next().ok_or("Missing source")?.into(),
            "--setup-only" => setup_only = true,
            _ => return err("Unknown installer option"),
        }
    }
    let source = source.canonicalize()?;
    let target = crate::home()?.join(".config/omarchy/plugins/lbiasuz.keycorner");
    run(
        "omarchy",
        &[
            "plugin",
            "validate",
            source.to_str().ok_or("Invalid source path")?,
        ],
    )?;
    if !setup_only {
        fs::create_dir_all(target.join("bin"))?;
        if source != target.canonicalize()? {
            for name in FILES.lines() {
                fs::create_dir_all(target.join(name).parent().unwrap())?;
                atomic(
                    &target.join(name),
                    &fs::read(source.join(name))?,
                    if name == "keycorner.sh" { 0o755 } else { 0o644 },
                )?;
            }
        }
        let executable = std::env::current_exe()?;
        let destination = target.join("bin/keycorner");
        if executable != destination {
            atomic(&destination, &fs::read(executable)?, 0o755)?;
        }
    }
    config_init()?;
    let hypr = crate::config_root()?.join("hypr");
    setup_include(
        &source,
        &hypr,
        "hyprland.lua",
        "keycorner.lua",
        "config/hyprland.lua",
        "require(\"hypr.keycorner\")",
        false,
    )?;
    setup_include(
        &source,
        &hypr,
        "bindings.lua",
        "keycorner-shortcuts.lua",
        "config/shortcuts.lua",
        "require(\"hypr.keycorner-shortcuts\")",
        true,
    )?;
    if !setup_only {
        let shell = crate::home()?.join(".config/omarchy/shell.json");
        if shell.exists() {
            atomic(
                &shell.with_extension(format!(
                    "json.keycorner-backup-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_nanos()
                )),
                &fs::read(&shell)?,
                0o600,
            )?;
        }
        run("omarchy-shell", &["shell", "rescanPlugins"])?;
        let mut found = false;
        for _ in 0..30 {
            if run("omarchy-shell", &["shell", "listPlugins"])?.contains("lbiasuz.keycorner") {
                found = true;
                break;
            };
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        if !found {
            return err("Plugin was not discovered; inspect Omarchy shell logs");
        };
        run("omarchy", &["plugin", "enable", "lbiasuz.keycorner"])?;
    }
    Ok(())
}
