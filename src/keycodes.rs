//! Vial's keycode encoding changed at protocol 6. The same number must not be
//! interpreted with the other protocol's table.
use crate::core;
use serde_json::Value;
fn modifiers(bits: u16) -> String {
    let names = if bits & 16 != 0 {
        ["RCtrl", "RShift", "AltGr", "RSuper"]
    } else {
        ["Ctrl", "Shift", "Alt", "Super"]
    };
    names
        .iter()
        .enumerate()
        .filter(|(i, _)| bits & (1 << i) != 0)
        .map(|(_, s)| *s)
        .collect::<Vec<_>>()
        .join("+")
}
pub fn label(code: u16, custom: &Value, protocol: u32) -> String {
    let legacy = protocol < 6;
    let custom_start = if legacy { 0x5f80 } else { 0x7e00 };
    if (custom_start..custom_start + 64).contains(&code) {
        let entry = &custom[(code - custom_start) as usize];
        if let Some(name) = ["shortName", "name", "title"]
            .iter()
            .find_map(|k| entry[*k].as_str().filter(|s| !s.is_empty()))
        {
            return name.chars().take(64).collect();
        };
        return format!("User {}", code - custom_start);
    }
    if code < 256 {
        let extra = match code {
            70 => Some("PrtSc"),
            71 => Some("Scroll"),
            72 => Some("Pause"),
            83 => Some("Num"),
            84 => Some("/"),
            85 => Some("*"),
            86 => Some("−"),
            87 => Some("+"),
            88 => Some("Enter"),
            98 => Some("0"),
            99 => Some("."),
            100 => Some("\\"),
            101 => Some("Menu"),
            102 => Some("Power"),
            103 => Some("="),
            _ => None,
        };
        if let Some(s) = extra {
            return s.into();
        };
        if (89..=97).contains(&code) {
            return (code - 88).to_string();
        };
        if legacy && (240..=248).contains(&code) {
            return match code {
                240 => "Mouse ↑".into(),
                241 => "Mouse ↓".into(),
                242 => "Mouse ←".into(),
                243 => "Mouse →".into(),
                _ => format!("Btn{}", code - 243),
            };
        }
        return core::label(code, &Value::Null);
    }
    let modtap = if legacy {
        0x6000..=0x7fff
    } else {
        0x2000..=0x3fff
    };
    if modtap.contains(&code) {
        return format!(
            "{}\n{}",
            label(code & 255, &Value::Null, protocol),
            modifiers((code >> 8) & 31)
        );
    }
    if (0x4000..=0x4fff).contains(&code) {
        return format!(
            "{}\nL{}",
            label(code & 255, &Value::Null, protocol),
            (code >> 8) & 15
        );
    }
    if (0x100..=0x1fff).contains(&code) {
        let base = label(code & 255, &Value::Null, protocol);
        if code >> 8 == 2
            && let Some(i) = "1234567890-=[]\\;'`,./"
                .chars()
                .position(|c| c.to_string() == base)
        {
            return "!@#$%^&*()_+{}|:\"~<>?".chars().nth(i).unwrap().to_string();
        };
        return format!("{}+{}", modifiers(code >> 8), base);
    }
    let ranges = if legacy {
        [
            (0x5100, "MO"),
            (0x5200, "DF"),
            (0x5300, "TG"),
            (0x5400, "OSL"),
            (0x5800, "TT"),
        ]
    } else {
        [
            (0x5220, "MO"),
            (0x5240, "DF"),
            (0x5260, "TG"),
            (0x5280, "OSL"),
            (0x52c0, "TT"),
        ]
    };
    for (start, name) in ranges {
        if (start..start + 32).contains(&code) {
            return format!("{name} {}", code - start);
        }
    }
    if legacy && (0x5010..=0x501f).contains(&code) {
        return format!("TO {}", code & 15);
    }
    if !legacy && (0x5200..=0x521f).contains(&code) {
        return format!("TO {}", code & 31);
    }
    if !legacy && (0x52e0..=0x52ff).contains(&code) {
        return format!("PDF {}", code & 31);
    }
    let osm = if legacy { 0x5500 } else { 0x52a0 };
    if (osm..osm + 32).contains(&code) {
        return format!("OSM {}", modifiers(code - osm));
    }
    if (0x5700..=0x57ff).contains(&code) {
        return format!("TD {}", code & 255);
    }
    // Preserve unknown actions as codes; do not invent their outputs.
    format!("{code:04X}")
}
pub fn usage(code: u16, protocol: u32) -> Option<u16> {
    let modtap = if protocol < 6 {
        0x6000..=0x7fff
    } else {
        0x2000..=0x3fff
    };
    if (0x100..=0x1fff).contains(&code)
        || (0x4000..=0x4fff).contains(&code)
        || modtap.contains(&code)
    {
        return core::usage(code & 255);
    }
    if code < 256 {
        return core::usage(code);
    }
    None
}
