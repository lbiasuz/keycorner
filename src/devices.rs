use crate::{Result, err, hid::Descriptor, run, string};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}
fn entries(path: &str, prefix: &str) -> Vec<PathBuf> {
    let mut out: Vec<_> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .map(|e| e.path())
        .collect();
    out.sort();
    out
}
fn usb(path: &Path) -> Option<PathBuf> {
    let resolved = path.canonicalize().ok()?;
    resolved
        .ancestors()
        .skip(1)
        .find(|p| p.join("idVendor").is_file() && p.join("idProduct").is_file())
        .map(Path::to_path_buf)
}
pub fn valid_id(s: &str) -> bool {
    s.len() == 24
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn accessible(p: &Value, mode: i32) -> bool {
    p.as_str()
        .and_then(|s| std::ffi::CString::new(s).ok())
        .is_some_and(|s| unsafe { libc::access(s.as_ptr(), mode) == 0 })
}
pub fn discover() -> Result<Vec<Value>> {
    let mut groups: BTreeMap<String, Value> = BTreeMap::new();
    for entry in entries("/sys/class/input", "event") {
        let bits = read(entry.join("device/capabilities/key"));
        let words: Vec<_> = bits.split_whitespace().collect();
        let last = words
            .last()
            .and_then(|s| u64::from_str_radix(s, 16).ok())
            .unwrap_or(0);
        if ![28, 30, 44, 42].iter().all(|k| last & (1u64 << k) != 0) {
            continue;
        }
        let node = format!(
            "/dev/input/{}",
            entry.file_name().unwrap().to_string_lossy()
        );
        let props = match run(
            "udevadm",
            &["info", "--query=property", &format!("--name={node}")],
        ) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !props.lines().any(|l| l == "ID_INPUT_KEYBOARD=1") {
            continue;
        }
        let parent = usb(&entry.join("device"));
        let mut name: String = read(entry.join("device/name")).chars().take(160).collect();
        if name.is_empty() {
            name = "Keyboard".into()
        }
        let (identity, matcher) = if let Some(p) = &parent {
            let vid = read(p.join("idVendor"));
            let pid = read(p.join("idProduct"));
            let serial = read(p.join("serial"));
            let product = read(p.join("product"));
            if !product.is_empty() {
                name = product.chars().take(160).collect()
            };
            (
                format!("usb:{}:{vid}:{pid}:{serial}", p.display()),
                json!({"usb_port":p.file_name().unwrap().to_string_lossy(),"vendor":vid,"product":pid,"serial":serial}),
            )
        } else {
            let phys = read(entry.join("device/phys"));
            let uniq = read(entry.join("device/uniq"));
            if phys.is_empty() {
                continue;
            };
            (
                format!("input:{name}:{phys}:{uniq}"),
                json!({"name":name,"phys":phys,"uniq":uniq}),
            )
        };
        let id = format!("{:x}", Sha256::digest(identity.as_bytes()))[..24].to_owned();
        let Ok(st) = fs::metadata(&node) else {
            continue;
        };
        let g=groups.entry(id.clone()).or_insert_with(||json!({"id":id,"name":name,"match":matcher,"events":[],"eventIdentity":{},"hidraw":[],"vial":false}));
        g["events"].as_array_mut().unwrap().push(json!(node));
        g["eventIdentity"][&node] = json!([st.dev(), st.ino(), st.rdev()]);
        if let Some(p) = parent {
            g["_usb"] = json!(p.to_string_lossy());
        }
    }
    for entry in entries("/sys/class/hidraw", "hidraw") {
        let Some(p) = usb(&entry.join("device")) else {
            continue;
        };
        for g in groups.values_mut() {
            if g["_usb"].as_str() == p.to_str() {
                g["hidraw"].as_array_mut().unwrap().push(json!(format!(
                    "/dev/{}",
                    entry.file_name().unwrap().to_string_lossy()
                )));
            }
        }
    }
    for g in groups.values_mut() {
        g.as_object_mut().unwrap().remove("_usb");
        let m = &g["match"];
        let interfaces = interfaces(g)?;
        let vial = interfaces.iter().any(|(_, d)| d.raw);
        let keyboard_reports = interfaces.iter().any(|(_, d)| d.keyboard());
        let conn = m
            .get("usb_port")
            .or_else(|| m.get("phys"))
            .cloned()
            .unwrap_or(json!(""));
        g["vial"] = json!(vial);
        g["keyboardReports"] = json!(keyboard_reports);
        g["connection"] = conn;
        g["standardAccessible"] = json!(
            g["events"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| accessible(p, libc::R_OK))
        );
        g["vialAccessible"] = json!(
            vial && keyboard_reports
                && interfaces
                    .iter()
                    .filter(|(_, d)| d.raw || d.readable())
                    .all(|(p, d)| accessible(
                        &json!(p),
                        if d.raw {
                            libc::R_OK | libc::W_OK
                        } else {
                            libc::R_OK
                        }
                    ))
        );
    }
    Ok(groups.into_values().collect())
}
pub fn select(id: &str) -> Result<Value> {
    if !valid_id(id) {
        return err("Select a keyboard in Settings");
    };
    discover()?
        .into_iter()
        .find(|d| d["id"] == id)
        .ok_or_else(|| "Selected keyboard disconnected; reconnect or choose a keyboard".into())
}
pub fn interfaces(device: &Value) -> Result<Vec<(String, Descriptor)>> {
    let mut out = Vec::new();
    for node in device["hidraw"].as_array().ok_or("Missing interfaces")? {
        let n = string(node)?;
        let name = Path::new(n).file_name().ok_or("Invalid node")?;
        let path = Path::new("/sys/class/hidraw")
            .join(name)
            .join("device/report_descriptor");
        // Unknown descriptors remain usable through Standard/evdev mode.
        if let Ok(bytes) = fs::read(path)
            && let Ok(desc) = Descriptor::parse(&bytes)
        {
            out.push((n.to_owned(), desc));
        }
    }
    Ok(out)
}
pub fn vial_path(interfaces: &[(String, Descriptor)]) -> Result<&str> {
    let raw: Vec<_> = interfaces.iter().filter(|(_, d)| d.raw).collect();
    if raw.len() != 1 {
        return err("Expected one Vial raw HID interface; choose Standard mode");
    };
    Ok(&raw[0].0)
}
