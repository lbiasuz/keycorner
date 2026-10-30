use super::*;
use crate::core::Keys;
use sha2::{Digest, Sha256};
fn fixture() -> Value {
    serde_json::from_str(include_str!("../tests/rust-golden.json")).unwrap()
}
#[test]
fn all_qmk_labels_and_usages_match_python() {
    let f = fixture();
    let mut h = Sha256::new();
    for code in 0..=65535u16 {
        h.update(
            format!(
                "{code}:{}:{}\n",
                core::label(code, &f["custom"]),
                core::usage(code)
                    .map(|u| u.to_string())
                    .unwrap_or("null".into())
            )
            .as_bytes(),
        );
    }
    assert_eq!(format!("{:x}", h.finalize()), f["labels_sha256"]);
}
#[test]
fn report_formats_match_python() {
    for case in fixture()["reports"].as_array().unwrap() {
        let data: Vec<_> = case["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let parsed = core::report(&data, case["kind"].as_str().unwrap());
        let v = parsed
            .map(|(group, keys)| json!([group, keys]))
            .unwrap_or(Value::Null);
        assert_eq!(v, case["expected"]);
    }
}
#[test]
fn geometry_and_layers_match_python() {
    let f = fixture();
    let codes: Vec<_> = f["codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u16)
        .collect();
    let layout = core::model(&f["definition"], 2, &codes).unwrap();
    assert_eq!(layout, f["model"]);
    assert_eq!(core::infer(&layout, &Keys::from([80])), Some(1));
    for pressed in [
        Keys::from([4]),
        Keys::from([224]),
        Keys::from([99]),
        Keys::new(),
    ] {
        assert_eq!(core::infer(&layout, &pressed), None);
    }
}
#[test]
fn malformed_layouts_fail_closed() {
    let f = fixture();
    let codes: Vec<_> = f["codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u16)
        .collect();
    for entry in [
        json!({"r":1}),
        json!({"x":129}),
        json!({"w":true}),
        json!({"w":-1}),
        json!("9,0"),
        json!("0,0,0"),
    ] {
        let mut d = f["definition"].clone();
        d["layouts"]["keymap"][0] = json!([entry, "0,0"]);
        assert!(core::model(&d, 2, &codes).is_err());
    }
    assert!(core::model(&f["definition"], 33, &codes).is_err());
    assert!(core::model(&f["definition"], 2, &[]).is_err());
}
#[test]
fn compressed_definition_limits() {
    use std::io::Write;
    let f = fixture();
    let bytes: Vec<u8> = f["compressed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    assert_eq!(core::decode(&bytes).unwrap(), f["definition"]);
    for n in [0, 1, bytes.len() / 2, bytes.len() - 1] {
        assert!(core::decode(&bytes[..n]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.extend(&bytes);
    assert!(core::decode(&trailing).is_err());
    let mut encoder = xz2::write::XzEncoder::new(Vec::new(), 6);
    encoder.write_all(&vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert!(core::decode(&encoder.finish().unwrap()).is_err());
}
#[test]
fn standard_presets() {
    for (p, count) in [("ansi104", 104), ("ansi87", 87), ("ansi60", 61)] {
        let layout = core::standard(p).unwrap();
        assert_eq!(layout["keys"].as_array().unwrap().len(), count);
        assert_eq!(layout["layers"], 1);
        assert_eq!(layout["source"], "standard");
    }
    assert!(core::standard("other").is_err());
}
fn event(kind: u16, code: u16, value: i32) -> Vec<u8> {
    let mut bytes = vec![0; std::mem::size_of::<libc::timeval>()];
    bytes.extend(kind.to_ne_bytes());
    bytes.extend(code.to_ne_bytes());
    bytes.extend(value.to_ne_bytes());
    bytes
}
#[test]
fn evdev_partial_repeat_drop_resync() {
    let mut reader = stream::Reader::default();
    let input = [event(1, 30, 1), event(1, 30, 2), event(0, 0, 0)].concat();
    assert!(
        reader
            .feed(&input[..7], || Ok(Keys::new()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        reader.feed(&input[7..], || Ok(Keys::new())).unwrap(),
        vec![Keys::from([30])]
    );
    let input = [
        event(0, 3, 0),
        event(1, 44, 1),
        event(0, 0, 0),
        event(1, 42, 0),
        event(0, 0, 0),
    ]
    .concat();
    assert_eq!(
        reader.feed(&input, || Ok(Keys::from([42]))).unwrap(),
        vec![Keys::new(), Keys::from([42]), Keys::new()]
    );
}
#[test]
fn shortcut_roundtrip_and_conflicts() {
    let f = fixture();
    assert_eq!(shortcuts::render(&f["shortcuts"]), f["rendered_shortcuts"]);
    assert_eq!(
        shortcuts::read_text(f["rendered_shortcuts"].as_str().unwrap()).unwrap(),
        f["shortcuts"]
    );
    assert_eq!(
        shortcuts::normalize(&json!("shift + super + Return")).unwrap(),
        "SUPER + SHIFT + Return"
    );
    for value in [
        json!("A"),
        json!("SUPER + SUPER + K"),
        json!("SUPER + x\")"),
        json!(true),
    ] {
        assert!(shortcuts::normalize(&value).is_err());
    }
    assert!(shortcuts::read_text("os.execute('evil')").is_err());
    let binds = json!([{"modmask":69,"key":"K","description":"Someone else"}]);
    assert!(shortcuts::conflicts(&f["shortcuts"], &json!({}), &binds).is_err());
    let own =
        json!([{"modmask":69,"key":"K","description":"Keycorner: show/resume or hide/pause"}]);
    assert!(shortcuts::conflicts(&f["shortcuts"], &f["shortcuts"], &own).is_ok());
}
#[test]
fn device_rules_match_and_reject_injection() {
    let f = fixture();
    for backend in ["standard", "vial"] {
        assert_eq!(
            permissions::rule(&f["device"], backend).unwrap(),
            f["rules"][backend]
        );
    }
    for value in ["*", "a\"b", "abc%k", "a\\b", "a\nb", "a?b", "a[b]", "a$b"] {
        let mut d = f["device"].clone();
        d["match"]["serial"] = json!(value);
        assert!(permissions::rule(&d, "standard").is_err());
    }
    let mut d = f["device"].clone();
    d["id"] = json!("../../etc/passwd");
    assert!(permissions::rule(&d, "standard").is_err());
    assert!(permissions::rule(&f["device"], "other").is_err());
}
#[test]
fn transport_rejects_writes_before_touching_device() {
    let mut file = std::fs::File::open("/dev/null").unwrap();
    for request in [
        vec![],
        vec![0; 33],
        vec![5],
        vec![19],
        vec![254],
        vec![254, 3],
        vec![254, 6],
        vec![254, 8],
    ] {
        let error = stream::query(&mut file, &request).unwrap_err();
        assert_eq!(error.to_string(), "Not a read-only Vial command");
    }
}
