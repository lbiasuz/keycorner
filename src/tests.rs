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
    for (p, count) in [
        ("ansi104", 104),
        ("ansi87", 87),
        ("ansi60", 61),
        ("abnt2", 107),
    ] {
        let layout = core::standard(p).unwrap();
        assert_eq!(layout["keys"].as_array().unwrap().len(), count);
        assert_eq!(layout["layers"], 1);
        assert_eq!(layout["source"], "standard");
    }
    assert!(core::standard("other").is_err());
}
#[test]
fn abnt2_distinct_physical_keys() {
    let layout = core::standard("abnt2").unwrap();
    let keys = layout["keys"].as_array().unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for key in keys {
        assert!(seen.insert(key["codes"][0].as_u64().unwrap()));
    }
    for (code, label) in [
        (39, "Ç"),
        (86, "\\"),
        (89, "/"),
        (100, "AltGr"),
        (121, "."),
        (83, ","),
    ] {
        let key = keys.iter().find(|k| k["codes"][0] == code).unwrap();
        assert_eq!(key["labels"][0], label);
    }
    // Rectangular approximations must not cover a neighboring switch.
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            let n = |k: &serde_json::Value, f: &str| k[f].as_f64().unwrap();
            assert!(
                n(a, "x") + n(a, "w") <= n(b, "x")
                    || n(b, "x") + n(b, "w") <= n(a, "x")
                    || n(a, "y") + n(a, "h") <= n(b, "y")
                    || n(b, "y") + n(b, "h") <= n(a, "y")
            );
        }
    }
}
#[test]
fn regional_standard_presets() {
    for region in ["gb", "de", "fr", "es", "latam", "it", "pt"] {
        for (suffix, count) in [("105", 105), ("88", 88)] {
            let layout = core::standard(&format!("{region}{suffix}")).unwrap();
            let keys = layout["keys"].as_array().unwrap();
            assert_eq!(keys.len(), count);
            assert!(keys.iter().any(|k| k["codes"][0] == 86));
            assert_eq!(keys.iter().any(|k| k["codes"][0] == 79), suffix == "105");
        }
    }
    for (preset, code, label) in [
        ("de105", 21, "Z"),
        ("de88", 44, "Y"),
        ("fr105", 16, "A"),
        ("fr88", 30, "Q"),
        ("es105", 39, "Ñ"),
        ("latam88", 40, "{"),
        ("pt105", 39, "Ç"),
        ("it88", 26, "è"),
        ("gb105", 43, "#"),
    ] {
        let layout = core::standard(preset).unwrap();
        assert_eq!(
            layout["keys"]
                .as_array()
                .unwrap()
                .iter()
                .find(|k| k["codes"][0] == code)
                .unwrap()["labels"][0],
            label
        );
    }
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
fn descriptor(hex: &str) -> crate::hid::Descriptor {
    let bytes: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    crate::hid::Descriptor::parse(&bytes).unwrap()
}
#[test]
fn descriptor_keyboard_reports_match_reference() {
    // Standard HID boot descriptor; no product or vendor identity participates.
    let boot = descriptor(
        "05010906a101050719e029e7150025019508750181029501750881010507190029ff150026ff00950675088100c0",
    );
    let shared = descriptor(
        "05010906a1018506050719e029e7150025019508750181020507190029ef1500250195f075018102c0050c0901a101850419012aa002150126a002950175108100c0",
    );
    assert!(boot.keyboard());
    assert!(shared.keyboard());
    assert!(!boot.raw);
    for case in fixture()["reports"].as_array().unwrap() {
        let data: Vec<_> = case["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let desc = if case["kind"] == "boot" {
            &boot
        } else {
            &shared
        };
        let parsed = desc.decode(&data);
        let actual = parsed
            .first()
            .map(|(group, keys)| json!([group, keys]))
            .unwrap_or(Value::Null);
        assert_eq!(actual, case["expected"]);
    }
}
#[test]
fn descriptor_dynamic_ids_padding_and_raw_transport() {
    let numbered =
        descriptor("05010906a1018521050719e029e795087501810295017508810195067508190029ff8100c0");
    assert_eq!(
        numbered.decode(&[33, 2, 0, 4, 5, 0, 0, 0, 0]),
        vec![("keyboard", Keys::from([225, 4, 5]))]
    );
    assert!(numbered.decode(&[6, 2, 0, 4, 5, 0, 0, 0, 0]).is_empty());
    let consumer = descriptor("050c0901a10185379501751019012aff028100c0");
    assert_eq!(
        consumer.decode(&[55, 0xe9, 0]),
        vec![("media", Keys::from([169]))]
    );
    assert!(!consumer.keyboard());
    let raw = descriptor("0660ff0961a1010962150026ff009520750881020963150026ff00952075089102c0");
    assert!(raw.raw);
    assert!(!raw.readable());
    assert!(raw.decode(&[0; 32]).is_empty());
    let fake = descriptor("0600ff0961a1019520750881029102c0");
    assert!(!fake.raw);
    let padded = descriptor("05010906a101950375018101050719e029e7950875018102950575018101c0");
    assert_eq!(
        padded.decode(&[0x10, 0]),
        vec![("keyboard", Keys::from([225]))]
    );
}
#[test]
fn malformed_hid_descriptors_fail_safely() {
    for bytes in [
        vec![0xfe],
        vec![0x76, 1],
        vec![0xc0],
        vec![0xb4],
        vec![0x85, 0],
        vec![0xa1, 1],
        vec![0x75, 32, 0x96, 255, 255, 0x81, 0],
    ] {
        assert!(crate::hid::Descriptor::parse(&bytes).is_err(), "{bytes:?}");
    }
    let unknown = descriptor("05010906a1010507190029ff750595018100c0");
    assert!(!unknown.keyboard());
    assert!(unknown.decode(&[1]).is_empty());
}
#[test]
fn permission_rules_accept_other_usb_keyboards_without_widening_scope() {
    let mut device = fixture()["device"].clone();
    device["match"]["vendor"] = json!("1234");
    device["match"]["product"] = json!("abcd");
    device["match"]["serial"] = json!("");
    let rule = permissions::rule(&device, "vial").unwrap();
    assert!(rule.contains("ATTRS{idVendor}==\"1234\""));
    assert!(rule.contains("ATTRS{idProduct}==\"abcd\""));
    assert!(rule.contains("KERNELS==\"1-6\""));
    assert!(rule.contains("TAG+=\"uaccess\""));
    assert!(!rule.contains("MODE="));
    assert!(!rule.contains("GROUP="));
}
#[test]
fn separate_keyboard_reports_preserve_each_others_state() {
    let desc = descriptor("05010906a1018521050719e029e7950875018102852295067508190029ff8100c0");
    let mut held = crate::hid::State::default();
    assert!(held.feed(&desc, &[33, 2]));
    assert!(held.feed(&desc, &[34, 4, 0, 0, 0, 0, 0]));
    assert_eq!(held.keys().collect::<Keys>(), Keys::from([225, 4]));
    assert!(!held.feed(&desc, &[34, 4, 0, 0, 0, 0, 0]));
    assert!(held.feed(&desc, &[34, 0, 0, 0, 0, 0, 0]));
    assert_eq!(held.keys().collect::<Keys>(), Keys::from([225]));
    assert!(held.feed(&desc, &[33, 0]));
    assert!(held.keys().next().is_none());
}
#[test]
fn vial_protocol_versions_decode_home_row_mods_and_layers() {
    for (legacy, modern, key, hold) in [
        (0x6804, 0x2804, 4, "Super"),
        (0x6416, 0x2416, 22, "Alt"),
        (0x7207, 0x3207, 7, "RShift"),
        (0x7109, 0x3109, 9, "RCtrl"),
    ] {
        for (code, protocol) in [(legacy, 5), (modern, 6)] {
            assert_eq!(
                keycodes::label(code, &Value::Null, protocol),
                format!("{}\n{hold}", core::label(key, &Value::Null))
            );
            assert_eq!(keycodes::usage(code, protocol), Some(key));
        }
        assert_eq!(keycodes::usage(legacy, 6), None);
        assert_eq!(keycodes::usage(modern, 5), None);
    }
    for (code, proto, expected) in [
        (0x5102, 5, "MO 2"),
        (0x5804, 5, "TT 4"),
        (0x5802, 5, "TT 2"),
        (0x5801, 5, "TT 1"),
        (0x5222, 6, "MO 2"),
        (0x52c4, 6, "TT 4"),
        (0x5202, 5, "DF 2"),
        (0x5202, 6, "TO 2"),
    ] {
        assert_eq!(keycodes::label(code, &Value::Null, proto), expected);
        assert_eq!(keycodes::usage(code, proto), None);
    }
    assert_eq!(keycodes::label(0x7833, &Value::Null, 5), ";\nRSuper");
    assert_eq!(keycodes::label(70, &Value::Null, 5), "PrtSc");
}
#[test]
fn protocol_model_uses_tap_outputs_and_custom_names() {
    let d = json!({"name":"Synthetic legacy board","matrix":{"rows":1,"cols":4},"layouts":{"keymap":[["0,0","0,1","0,2","0,3"]]}});
    let model = core::model_protocol(&d, 1, &[0x6804, 0x6416, 0x7207, 0x7109], 5).unwrap();
    assert_eq!(
        model["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| k["usages"][0].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [4, 22, 7, 9]
    );
    let custom =
        json!([{"name":"CUSTOM_ACTION","shortName":"Action","title":"Action description"}]);
    for (code, protocol) in [(0x5f80, 5), (0x7e00, 6)] {
        assert_eq!(keycodes::label(code, &custom, protocol), "Action");
        assert_eq!(keycodes::usage(code, protocol), None);
    }
    assert_eq!(keycodes::label(0x5f81, &custom, 5), "User 1");
}

#[test]
fn vial_layout_name_is_optional() {
    let definition =
        serde_json::json!({"matrix":{"rows":1,"cols":1},"layouts":{"keymap":[["0,0"]]}});
    let layout = crate::core::model_protocol(&definition, 1, &[4], 6).unwrap();
    assert_eq!(layout["name"], "Vial keyboard");
    assert_eq!(layout["keys"][0]["labels"][0], "A");
}

#[test]
fn mouse_buttons_ignore_motion_and_preserve_other_reports() {
    let mouse = descriptor(
        "05010902a10185020901a100050919012908150025017501950881020501093009311581257f750895028106c0c0",
    );
    assert!(!mouse.keyboard());
    assert!(mouse.readable());
    let mut state = crate::hid::State::default();
    assert!(state.feed(&mouse, &[2, 0x81, 0, 0]));
    assert_eq!(state.keys().collect::<Keys>(), Keys::from([257, 264]));
    assert!(!state.feed(&mouse, &[2, 0x81, 50, 255]));
    assert!(!state.feed(&mouse, &[2, 0])); // Truncated report cannot release a held button.
    assert!(state.feed(&mouse, &[2, 0, 0, 0]));
    assert!(state.keys().next().is_none());
    for button in 1..=8 {
        assert_eq!(crate::keycodes::usage(0xd0 + button, 6), Some(256 + button));
        assert_eq!(
            crate::keycodes::label(0xd0 + button, &Value::Null, 6),
            format!("Btn{button}")
        );
    }
    for button in 1..=5 {
        assert_eq!(crate::keycodes::usage(243 + button, 5), Some(256 + button));
    }
    let keyboard = descriptor("05010906a1018501050719e029e7950875018102c0");
    assert!(state.feed(&keyboard, &[1, 2]));
    assert!(state.feed(&mouse, &[2, 1, 0, 0]));
    assert_eq!(state.keys().collect::<Keys>(), Keys::from([225, 257]));
    assert!(state.feed(&mouse, &[2, 0, 0, 0]));
    assert_eq!(state.keys().collect::<Keys>(), Keys::from([225]));
}
