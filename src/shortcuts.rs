use crate::{Result, atomic, err, run, string};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};
pub const ACTIONS: [(&str, &str); 3] = [
    ("toggle", "Keycorner: show/resume or hide/pause"),
    ("toggleSettings", "Keycorner: toggle settings"),
    ("pause", "Keycorner: pause/resume input"),
];
const MODS: [(&str, u64); 4] = [("SUPER", 64), ("CTRL", 4), ("ALT", 8), ("SHIFT", 1)];
pub fn normalize(v: &Value) -> Result<String> {
    let s = string(v)?;
    if s.chars().count() > 100 {
        return err("Invalid shortcut");
    };
    if s.trim().is_empty() {
        return Ok(String::new());
    };
    let parts: Vec<_> = s.split('+').map(|p| p.trim().to_uppercase()).collect();
    if parts.len() < 2 {
        return err("Use modifiers plus a key");
    };
    let mods = &parts[..parts.len() - 1];
    if mods.iter().any(|p| !MODS.iter().any(|(m, _)| m == p))
        || mods.iter().collect::<BTreeSet<_>>().len() != mods.len()
    {
        return err("Invalid shortcut modifiers");
    };
    let upper = parts.last().unwrap();
    if upper.is_empty()
        || !upper
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || MODS.iter().any(|(m, _)| m == upper)
    {
        return err("Invalid shortcut key");
    };
    let mut key = s.split('+').next_back().unwrap().trim().to_owned();
    if key.len() == 1
        || key.starts_with(['f', 'F'])
            && key.len() > 1
            && key[1..].bytes().all(|b| b.is_ascii_digit())
    {
        key = key.to_uppercase()
    };
    let mut out: Vec<_> = MODS
        .iter()
        .filter(|(m, _)| mods.iter().any(|p| p == m))
        .map(|(m, _)| m.to_string())
        .collect();
    out.push(key);
    Ok(out.join(" + "))
}
fn identity(v: &str) -> (u64, String) {
    let p: Vec<_> = v.split(" + ").collect();
    let mask = p[..p.len() - 1]
        .iter()
        .filter_map(|p| MODS.iter().find(|(m, _)| m == p).map(|(_, n)| n))
        .sum();
    (mask, p.last().unwrap().to_uppercase())
}
pub fn read_text(text: &str) -> Result<Value> {
    let mut result = json!({"toggle":"","toggleSettings":"","pause":""});
    let mut seen = BTreeSet::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
    {
        let content = line
            .strip_prefix("o.bind(\"")
            .and_then(|s| s.strip_suffix("\")"))
            .ok_or("Shortcut file contains custom Lua")?;
        let parts: Vec<_> = content.split("\", \"").collect();
        if parts.len() != 3 {
            return err("Shortcut file contains custom Lua");
        };
        let action = parts[2]
            .strip_prefix("omarchy-shell keycorner ")
            .ok_or("Shortcut file contains custom Lua")?;
        if !ACTIONS.iter().any(|(a, d)| *a == action && *d == parts[1]) || !seen.insert(action) {
            return err("Shortcut file contains custom Lua");
        };
        result[action] = json!(normalize(&json!(parts[0]))?);
    }
    Ok(result)
}
pub fn render(values: &Value) -> String {
    let mut out="-- Managed by Keycorner settings. Blank shortcuts are disabled.\n-- Bindings are unavailable while the screen is locked.\n".to_owned();
    for (a, d) in ACTIONS {
        let value = values[a].as_str().unwrap_or("");
        if !value.is_empty() {
            out += &format!("o.bind(\"{value}\", \"{d}\", \"omarchy-shell keycorner {a}\")\n");
        }
    }
    out
}
pub fn conflicts(values: &Value, old: &Value, binds: &Value) -> Result<()> {
    let selected: Vec<_> = ACTIONS
        .iter()
        .filter_map(|(a, _)| values[*a].as_str().filter(|s| !s.is_empty()).map(identity))
        .collect();
    if selected.iter().collect::<BTreeSet<_>>().len() != selected.len() {
        return err("Each action needs a different shortcut");
    };
    for b in binds.as_array().ok_or("Invalid binding list")? {
        let combo = (
            b["modmask"].as_u64().unwrap_or(0),
            b["key"].as_str().unwrap_or("").to_uppercase(),
        );
        let owned = ACTIONS.iter().any(|(a, d)| {
            old[*a]
                .as_str()
                .filter(|s| !s.is_empty())
                .is_some_and(|s| identity(s) == combo)
                && b["description"] == *d
        });
        if selected.contains(&combo) && !owned {
            return err(&format!(
                "Shortcut already used by {}",
                b["description"]
                    .as_str()
                    .or(b["dispatcher"].as_str())
                    .unwrap_or("another action")
            ));
        }
    }
    Ok(())
}
pub fn validate() -> Result<()> {
    run("hyprctl", &["reload"])?;
    let e = run("hyprctl", &["configerrors"])?;
    if !e.trim().is_empty() && e.trim() != "ok" {
        return err(&format!("Hyprland rejected configuration: {e}"));
    };
    Ok(())
}
fn apply(path: &Path, requested: &Value) -> Result<Value> {
    let obj = requested
        .as_object()
        .ok_or("Expected all three shortcut actions")?;
    if obj.len() != 3 || !ACTIONS.iter().all(|(a, _)| obj.contains_key(*a)) {
        return err("Expected all three shortcut actions");
    };
    let mut values = json!({});
    for (a, _) in ACTIONS {
        values[a] = json!(normalize(&requested[a])?);
    }
    let original = fs::read_to_string(path)?;
    let old = read_text(&original)?;
    conflicts(
        &values,
        &old,
        &serde_json::from_str(&run("hyprctl", &["-j", "binds"])?)?,
    )?;
    let rendered = render(&values);
    if original != rendered {
        atomic(
            &path.with_extension("lua.backup"),
            original.as_bytes(),
            0o600,
        )?;
        if let Err(error) = atomic(path, rendered.as_bytes(), 0o600).and_then(|_| validate()) {
            atomic(path, original.as_bytes(), 0o600)?;
            validate()?;
            return Err(error);
        }
    }
    Ok(values)
}
pub fn command(args: &[String]) -> Result<Value> {
    let path = crate::config_root()?.join("hypr/keycorner-shortcuts.lua");
    let values = if args.is_empty() {
        read_text(&fs::read_to_string(path)?)?
    } else if args.len() == 2 && args[0] == "apply" {
        apply(&path, &serde_json::from_str(&args[1])?)?
    } else {
        return err("Invalid shortcuts command");
    };
    Ok(json!({"ok":true,"shortcuts":values}))
}
