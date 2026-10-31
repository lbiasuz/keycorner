//! Bounded decoding of keyboard/consumer/mouse-button HID input fields, using advertised report IDs.
//! Vendor fields are never interpreted as keys. No device I/O occurs here.
use crate::{Result, core::Keys, err};
use std::collections::BTreeMap;
#[derive(Clone, Default)]
struct Globals {
    page: u32,
    size: usize,
    count: usize,
    id: u8,
}
#[derive(Clone)]
struct Field {
    offset: usize,
    size: usize,
    count: usize,
    page: u32,
    variable: bool,
    usages: Vec<u16>,
}
#[derive(Clone, Default)]
struct Report {
    bits: usize,
    fields: Vec<Field>,
    unsupported: bool,
}
#[derive(Clone, Default)]
pub struct Descriptor {
    reports: BTreeMap<u8, Report>,
    pub raw: bool,
}
impl Descriptor {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() > 65536 {
            return err("HID descriptor too large");
        }
        let mut g = Globals::default();
        let mut globals = Vec::new();
        let mut collections = Vec::new();
        let mut app = (0, 0);
        let mut usages = Vec::<u16>::new();
        let (mut min, mut max) = (None, None);
        let mut out = Self::default();
        let (mut raw_input, mut raw_output) = (false, false);
        let mut i = 0;
        let mut fields = 0usize;
        let mut output_bits = BTreeMap::<u8, usize>::new();
        while i < data.len() {
            let prefix = data[i];
            i += 1;
            if prefix == 254 {
                return err("Unsupported long HID item");
            };
            let size = match prefix & 3 {
                3 => 4,
                n => n as usize,
            };
            if i + size > data.len() {
                return err("Truncated HID descriptor");
            };
            let mut bytes = [0; 4];
            bytes[..size].copy_from_slice(&data[i..i + size]);
            i += size;
            let value = u32::from_le_bytes(bytes);
            let kind = (prefix >> 2) & 3;
            let tag = prefix >> 4;
            match (kind, tag) {
                (1, 0) => g.page = value,
                (1, 7) => g.size = value as usize,
                (1, 8) => {
                    if !(1..=255).contains(&value) {
                        return err("Invalid HID report ID");
                    };
                    g.id = value as u8;
                }
                (1, 9) => g.count = value as usize,
                (1, 10) => {
                    if globals.len() >= 32 {
                        return err("HID globals too deep");
                    };
                    globals.push(g.clone());
                }
                (1, 11) => g = globals.pop().ok_or("Unbalanced HID globals")?,
                (2, 0) => {
                    if usages.len() >= 768 {
                        return err("Too many HID usages");
                    };
                    if value > 65535 {
                        return err("Extended HID usages are unsupported");
                    }
                    usages.push(value as u16);
                }
                (2, 1) => min = Some(value),
                (2, 2) => max = Some(value),
                (0, 10) => {
                    if collections.len() >= 32 {
                        return err("HID collections too deep");
                    };
                    collections.push(app);
                    if value == 1 {
                        app = (g.page, usages.first().copied().unwrap_or(0) as u32);
                    }
                }
                (0, 12) => app = collections.pop().ok_or("Unbalanced HID collection")?,
                (0, 8) | (0, 9) => {
                    let bits = g
                        .size
                        .checked_mul(g.count)
                        .filter(|b| *b <= 4096)
                        .ok_or("HID report too large")?;
                    if app == (0xff60, 0x61)
                        && g.id == 0
                        && g.size == 8
                        && g.count == 32
                        && value & 1 == 0
                    {
                        if tag == 8 {
                            raw_input = true
                        } else {
                            raw_output = true
                        }
                    }
                    if tag == 9 {
                        let total = output_bits.entry(g.id).or_default();
                        *total = total
                            .checked_add(bits)
                            .filter(|b| *b <= 4096)
                            .ok_or("HID output report too large")?;
                    }
                    if tag == 8 {
                        let report = out.reports.entry(g.id).or_default();
                        let offset = report.bits;
                        report.bits = report
                            .bits
                            .checked_add(bits)
                            .filter(|b| *b <= 4096)
                            .ok_or("HID report too large")?;
                        if value & 1 == 0
                            && ((app == (1, 6) && g.page == 7)
                                || (app == (12, 1) && g.page == 12)
                                || (app == (1, 2) && g.page == 9))
                        {
                            let variable = value & 2 != 0;
                            let mut mapped = usages.clone();
                            if mapped.is_empty()
                                && let (Some(lo), Some(hi)) = (min, max)
                                && lo <= hi
                                && hi <= 65535
                                && hi - lo < 768
                            {
                                mapped = (lo..=hi).map(|u| u as u16).collect();
                            }
                            if (variable && g.size == 1 && mapped.len() >= g.count)
                                || (!variable && matches!(g.size, 8 | 16))
                            {
                                if g.count == 0 {
                                    return err("Empty HID input field");
                                }
                                fields += 1;
                                if fields > 512 {
                                    return err("Too many HID input fields");
                                }
                                report.fields.push(Field {
                                    offset,
                                    size: g.size,
                                    count: g.count,
                                    page: g.page,
                                    variable,
                                    usages: mapped,
                                });
                            } else {
                                report.unsupported = true;
                            }
                        }
                    }
                }
                _ => (),
            }
            if kind == 0 {
                usages.clear();
                min = None;
                max = None;
            }
        }
        if !collections.is_empty() || !globals.is_empty() {
            return err("Unbalanced HID descriptor");
        };
        if out.reports.contains_key(&0) && out.reports.len() > 1 {
            return err("Mixed numbered and unnumbered reports");
        };
        out.raw = raw_input
            && raw_output
            && out.reports.get(&0).is_some_and(|r| r.bits == 256)
            && output_bits.get(&0) == Some(&256);
        Ok(out)
    }
    pub fn keyboard(&self) -> bool {
        self.reports
            .values()
            .any(|r| !r.unsupported && r.fields.iter().any(|f| f.page == 7))
    }
    pub fn readable(&self) -> bool {
        self.reports
            .values()
            .any(|r| !r.unsupported && !r.fields.is_empty())
    }
    pub fn decode(&self, data: &[u8]) -> Vec<(&'static str, Keys)> {
        let numbered = !self.reports.contains_key(&0);
        let (id, bytes) = if numbered {
            let Some((id, tail)) = data.split_first() else {
                return vec![];
            };
            (*id, tail)
        } else {
            (0, data)
        };
        let Some(report) = self.reports.get(&id) else {
            return vec![];
        };
        if report.unsupported || bytes.len() != report.bits.div_ceil(8) {
            return vec![];
        };
        let mut groups = BTreeMap::<&str, Keys>::new();
        for field in &report.fields {
            let name = match field.page {
                7 => "keyboard",
                9 => "mouse",
                _ => "media",
            };
            let held = groups.entry(name).or_default();
            for n in 0..field.count {
                let bit = field.offset + n * field.size;
                let mut value = 0u16;
                for b in 0..field.size {
                    if bytes[(bit + b) / 8] & (1 << ((bit + b) % 8)) != 0 {
                        value |= 1 << b;
                    }
                }
                let usage = if field.variable {
                    if value == 0 {
                        continue;
                    };
                    field.usages[n]
                } else {
                    value
                };
                if field.page == 7 {
                    if (4..=255).contains(&usage) {
                        held.insert(usage);
                    }
                } else if field.page == 9 {
                    if (1..=8).contains(&usage) {
                        held.insert(crate::keycodes::MOUSE_USAGE_BASE + usage);
                    }
                } else if let Some(code) = match usage {
                    0xe2 => Some(168),
                    0xe9 => Some(169),
                    0xea => Some(170),
                    0xb5 => Some(171),
                    0xb6 => Some(172),
                    0xb7 => Some(173),
                    0xcd => Some(174),
                    _ => None,
                } {
                    held.insert(code);
                }
            }
        }
        groups.into_iter().collect()
    }
}

/// Each report describes its own fields, not the whole keyboard. Preserve other
/// report states when modifiers, keys or consumer buttons arrive separately.
#[derive(Default)]
pub struct State {
    held: BTreeMap<(u8, &'static str), Keys>,
}
impl State {
    pub fn feed(&mut self, descriptor: &Descriptor, data: &[u8]) -> bool {
        let id = if descriptor.reports.contains_key(&0) {
            0
        } else {
            data.first().copied().unwrap_or(0)
        };
        let mut changed = false;
        for (group, keys) in descriptor.decode(data) {
            let current = self.held.entry((id, group)).or_default();
            if *current != keys {
                *current = keys;
                changed = true;
            }
        }
        changed
    }
    pub fn keys(&self) -> impl Iterator<Item = u16> + '_ {
        self.held.values().flat_map(|keys| keys.iter().copied())
    }
}
