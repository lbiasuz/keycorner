use crate::{Result, err, string};
use serde_json::{Value, json};
use std::{collections::BTreeSet, io::Read, sync::OnceLock};
pub type Keys = BTreeSet<u16>;
pub fn standard(preset: &str) -> Result<Value> {
    Ok(serde_json::from_str(match preset {
        "abnt2" => include_str!("../data/abnt2.json"),
        "ansi104" => include_str!("../data/ansi104.json"),
        "ansi87" => include_str!("../data/ansi87.json"),
        "ansi60" => include_str!("../data/ansi60.json"),
        "gb105" => include_str!("../data/gb105.json"),
        "gb88" => include_str!("../data/gb88.json"),
        "de105" => include_str!("../data/de105.json"),
        "de88" => include_str!("../data/de88.json"),
        "fr105" => include_str!("../data/fr105.json"),
        "fr88" => include_str!("../data/fr88.json"),
        "es105" => include_str!("../data/es105.json"),
        "es88" => include_str!("../data/es88.json"),
        "latam105" => include_str!("../data/latam105.json"),
        "latam88" => include_str!("../data/latam88.json"),
        "it105" => include_str!("../data/it105.json"),
        "it88" => include_str!("../data/it88.json"),
        "pt105" => include_str!("../data/pt105.json"),
        "pt88" => include_str!("../data/pt88.json"),
        _ => return err("Unsupported standard layout"),
    })?)
}
pub fn decode(data: &[u8]) -> Result<Value> {
    let stream = xz2::stream::Stream::new_auto_decoder(32 * 1024 * 1024, 0)?;
    let mut decoder = xz2::read::XzDecoder::new_stream(data, stream);
    let mut raw = Vec::new();
    decoder
        .by_ref()
        .take(1024 * 1024 + 1)
        .read_to_end(&mut raw)?;
    if raw.len() > 1024 * 1024 {
        return err("Keyboard definition exceeds size limit");
    }
    if decoder.total_in() != data.len() as u64 {
        return err("Unexpected trailing compressed data");
    }
    let v: Value = serde_json::from_slice(&raw)?;
    if !v.is_object() {
        return err("Keyboard definition must be an object");
    };
    Ok(v)
}
pub fn label(code: u16, custom: &Value) -> String {
    static BASIC: OnceLock<Value> = OnceLock::new();
    let basic = BASIC.get_or_init(|| {
        serde_json::from_str(include_str!("../data/labels.json")).expect("Static labels")
    });
    if let Some(s) = basic[code.to_string()].as_str() {
        return s.into();
    }
    if code >= 0x7e00
        && let Some(name) = custom[(code - 0x7e00) as usize].get("name")
    {
        return name
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| name.to_string())
            .chars()
            .take(64)
            .collect();
    }
    let base = || label(code & 255, &Value::Null);
    let mods = || {
        ["Ctrl", "Shift", "Alt", "Super"]
            .iter()
            .enumerate()
            .filter(|(i, _)| (code >> 8) & (1 << i) != 0)
            .map(|(_, m)| *m)
            .collect::<Vec<_>>()
            .join("+")
    };
    match code {
        0x4000..=0x4fff => format!("{}\nL{}", base(), (code >> 8) & 15),
        0x2000..=0x3fff => format!("{}\n{}", base(), mods()),
        0x100..=0x1fff => {
            let b = base();
            if code >> 8 == 2
                && let Some(i) = "1234567890-=[]\\;'`,./"
                    .chars()
                    .position(|c| c.to_string() == b)
            {
                return "!@#$%^&*()_+{}|:\"~<>?".chars().nth(i).unwrap().to_string();
            }
            format!("{}+{}", mods(), b)
        }
        0x5700..=0x57ff => format!("TD {}", code & 255),
        _ => {
            for (prefix, name) in [
                (0x5200, "TO"),
                (0x5220, "MO"),
                (0x5240, "DF"),
                (0x5260, "TG"),
                (0x5280, "OSL"),
                (0x52c0, "TT"),
            ] {
                if (prefix..prefix + 32).contains(&code) {
                    return format!("{name} {}", code & 31);
                }
            }
            format!("{code:04X}")
        }
    }
}
pub fn usage(code: u16) -> Option<u16> {
    match code {
        0x100..=0x4fff => Some(code & 255),
        4..=255 => Some(code),
        _ => None,
    }
}
fn number(v: &Value) -> Result<f64> {
    let n = v.as_f64().ok_or("Invalid key geometry")?;
    if !n.is_finite() || n.abs() > 128. {
        return err("Invalid key geometry");
    };
    Ok(n)
}
pub fn dimensions(d: &Value) -> Result<(usize, usize)> {
    let rows = d["matrix"]["rows"].as_u64().ok_or("Invalid rows")? as usize;
    let cols = d["matrix"]["cols"].as_u64().ok_or("Invalid cols")? as usize;
    if !(1..=32).contains(&rows) || !(1..=32).contains(&cols) {
        return err("Invalid matrix dimensions");
    };
    Ok((rows, cols))
}
#[cfg(test)]
pub fn model(d: &Value, layers: usize, codes: &[u16]) -> Result<Value> {
    model_inner(d, layers, codes, None)
}
pub fn model_protocol(d: &Value, layers: usize, codes: &[u16], protocol: u32) -> Result<Value> {
    model_inner(d, layers, codes, Some(protocol))
}
fn model_inner(d: &Value, layers: usize, codes: &[u16], protocol: Option<u32>) -> Result<Value> {
    let (rows, cols) = dimensions(d)?;
    if !(1..=32).contains(&layers)
        || layers * rows * cols * 2 > 65535
        || codes.len() != layers * rows * cols
    {
        return err("Invalid keymap size");
    }
    let raw = d["layouts"]["keymap"]
        .as_array()
        .ok_or("Invalid physical layout")?;
    if raw.len() > 128 {
        return err("Invalid physical layout");
    }
    let mut keys = Vec::new();
    let mut y = 0.;
    for row in raw {
        let Some(row) = row.as_array() else { continue };
        if row.len() > 256 {
            return err("Too many layout entries");
        };
        let (mut x, mut w, mut h) = (0., 1., 1.);
        for entry in row {
            if let Some(obj) = entry.as_object() {
                if ["r", "rx", "ry"].iter().any(|k| obj.contains_key(*k)) {
                    return err("Rotated KLE layouts are not supported yet");
                }
                for k in ["x", "y", "w", "h"] {
                    if let Some(v) = obj.get(k) {
                        number(v)?;
                    }
                }
                x += obj.get("x").map(number).transpose()?.unwrap_or(0.);
                y += obj.get("y").map(number).transpose()?.unwrap_or(0.);
                w = obj.get("w").map(number).transpose()?.unwrap_or(w);
                h = obj.get("h").map(number).transpose()?.unwrap_or(h);
            } else {
                let s = string(entry)?;
                if s.chars().count() > 256 {
                    return err("Invalid matrix position");
                };
                let pos: Vec<_> = s.split('\n').next().unwrap_or("").split(',').collect();
                if pos.len() != 2 {
                    return err("Invalid matrix position");
                };
                let r: usize = pos[0].trim().parse()?;
                let c: usize = pos[1].trim().parse()?;
                if r >= rows || c >= cols {
                    return err("Key is outside the declared matrix");
                };
                if !(w > 0. && w <= 16. && h > 0. && h <= 16. && x.abs() <= 128. && y.abs() <= 128.)
                {
                    return err("Invalid key bounds");
                };
                if keys.len() >= 1024 {
                    return err("Too many keys");
                }
                let values: Vec<_> = (0..layers)
                    .map(|l| codes[l * rows * cols + r * cols + c])
                    .collect();
                let effective: Vec<_> = values
                    .iter()
                    .map(|v| if *v == 1 { values[0] } else { *v })
                    .collect();
                keys.push(json!({"id":format!("{r},{c}"),"row":r,"col":c,"x":x,"y":y,"w":w,"h":h,"labels":effective.iter().map(|v|protocol.map(|p|crate::keycodes::label(*v,&d["customKeycodes"],p)).unwrap_or_else(||label(*v,&d["customKeycodes"]))).collect::<Vec<_>>(),"usages":effective.iter().map(|v|protocol.map(|p|crate::keycodes::usage(*v,p)).unwrap_or_else(||usage(*v))).collect::<Vec<_>>(),"codes":values}));
                x += w;
                w = 1.;
                h = 1.;
            }
        }
        y += 1.;
    }
    if keys.is_empty() {
        return err("Keyboard layout is empty");
    };
    let min_x = keys
        .iter()
        .map(|k| k["x"].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min);
    let min_y = keys
        .iter()
        .map(|k| k["y"].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min);
    let (mut width, mut height) = (0_f64, 0_f64);
    for k in &mut keys {
        k["x"] = json!(k["x"].as_f64().unwrap() - min_x);
        k["y"] = json!(k["y"].as_f64().unwrap() - min_y);
        width = width.max(k["x"].as_f64().unwrap() + k["w"].as_f64().unwrap());
        height = height.max(k["y"].as_f64().unwrap() + k["h"].as_f64().unwrap());
    }
    Ok(
        json!({"name":d["name"].as_str().filter(|s| !s.is_empty()).unwrap_or("Vial keyboard"),"layers":layers,"keys":keys,"width":width,"height":height}),
    )
}
#[cfg(test)]
pub fn report(data: &[u8], kind: &str) -> Option<(&'static str, Keys)> {
    let mut keys = Keys::new();
    let mods = if kind == "boot" && data.len() == 8 {
        keys.extend(data[2..].iter().map(|b| *b as u16).filter(|v| *v > 3));
        data[0]
    } else if kind == "shared" && data.len() == 32 && data[0] == 6 {
        for (i, byte) in data[2..].iter().enumerate() {
            for bit in 0..8 {
                let code = (i * 8 + bit) as u16;
                if byte & (1 << bit) != 0 && code > 3 {
                    keys.insert(code);
                }
            }
        }
        data[1]
    } else if kind == "shared" && data.len() == 3 && data[0] == 4 {
        let value = u16::from_le_bytes([data[1], data[2]]);
        if let Some((_, code)) = [
            (0xe2, 168),
            (0xe9, 169),
            (0xea, 170),
            (0xb5, 171),
            (0xb6, 172),
            (0xb7, 173),
            (0xcd, 174),
        ]
        .iter()
        .find(|(v, _)| *v == value)
        {
            keys.insert(*code);
        }
        return Some(("media", keys));
    } else {
        return None;
    };
    for i in 0..8 {
        if mods & (1 << i) != 0 {
            keys.insert(224 + i);
        }
    }
    Some(("keyboard", keys))
}
pub fn infer(layout: &Value, pressed: &Keys) -> Option<usize> {
    let meaningful: Keys = pressed
        .iter()
        .copied()
        .filter(|v| !(224..232).contains(v))
        .collect();
    if meaningful.is_empty() {
        return None;
    };
    let candidates: Vec<_> = (0..layout["layers"].as_u64()? as usize)
        .filter(|l| {
            let usages: Keys = layout["keys"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| k["usages"][*l].as_u64().map(|v| v as u16))
                .collect();
            meaningful.is_subset(&usages)
        })
        .collect();
    if candidates.len() == 1 {
        Some(candidates[0])
    } else {
        None
    }
}
