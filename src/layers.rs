//! Optional Keycorner layer reports on the existing Vial raw interface.
use crate::{Result, keycodes};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Read, Write},
    os::fd::AsRawFd,
    time::{Duration, Instant},
};

const MAGIC: &[u8; 6] = b"\xfcKCLY\x01";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub layers: u32,
    pub default: u32,
}
impl State {
    pub fn mask(self) -> u32 {
        let m = self.layers | self.default;
        if m == 0 { 1 } else { m }
    }
    pub fn highest(self) -> u32 {
        31 - self.mask().leading_zeros()
    }
}
fn request(op: u8, token: u32) -> [u8; 33] {
    let mut b = [0; 33];
    b[1..7].copy_from_slice(MAGIC);
    b[7] = op;
    b[9..13].copy_from_slice(&token.to_le_bytes());
    b
}
pub fn parse(b: &[u8], token: u32, count: usize) -> Option<State> {
    if b.len() != 32
        || &b[..6] != MAGIC
        || !matches!(b[6], 0x80 | 0x81 | 0x83)
        || b[7] != 1
        || b[8..12] != token.to_le_bytes()
        || !(1..=32).contains(&count)
    {
        return None;
    }
    let s = State {
        layers: u32::from_le_bytes(b[12..16].try_into().ok()?),
        default: u32::from_le_bytes(b[16..20].try_into().ok()?),
    };
    let valid = u32::MAX >> (32 - count);
    if s.mask() & !valid != 0 {
        None
    } else {
        Some(s)
    }
}
pub struct Live {
    pub file: File,
    token: u32,
    renewed: Instant,
    received: Instant,
    pub state: State,
    count: usize,
}
impl Live {
    pub fn connect(mut file: File, count: usize) -> Result<Option<Self>> {
        let mut nonce = [0; 4];
        File::open("/dev/urandom")?.read_exact(&mut nonce)?;
        let token = u32::from_le_bytes(nonce);
        if file.write(&request(0, token))? != 33 {
            return Ok(None);
        }
        if crate::stream::poll(&[file.as_raw_fd()], 250)?.is_empty() {
            return Ok(None);
        }
        let mut b = [0; 32];
        let n = file.read(&mut b)?;
        let Some(state) = parse(&b[..n], token, count) else {
            return Ok(None);
        };
        let mut live = Self {
            file,
            token,
            renewed: Instant::now(),
            received: Instant::now(),
            state,
            count,
        };
        live.send(1)?;
        Ok(Some(live))
    }
    fn send(&mut self, op: u8) -> Result<()> {
        if self.file.write(&request(op, self.token))? != 33 {
            return crate::err("Short layer request");
        }
        Ok(())
    }
    pub fn timeout(&self) -> i32 {
        Duration::from_secs(1)
            .saturating_sub(self.renewed.elapsed())
            .as_millis()
            .min(1000) as i32
    }
    pub fn renew(&mut self) -> Result<()> {
        if self.received.elapsed() >= Duration::from_secs(3) {
            return crate::err("Layer reports timed out; reconnecting");
        }
        if self.renewed.elapsed() >= Duration::from_secs(1) {
            self.send(1)?;
            self.renewed = Instant::now();
        }
        Ok(())
    }
    pub fn receive(&mut self) -> Result<bool> {
        let mut changed = false;
        for _ in 0..256 {
            let mut b = [0; 64];
            let n = match self.file.read(&mut b) {
                Ok(0) => return crate::err("Layer interface disconnected"),
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            };
            if let Some(state) = parse(&b[..n], self.token, self.count) {
                changed |= state != self.state;
                self.state = state;
                self.received = Instant::now();
            }
        }
        Ok(changed)
    }
}
impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.send(2);
    }
}
/// Transparent keys fall through enabled layers, rather than all numeric layers.
pub fn effective(codes: &[Value], mask: u32) -> u16 {
    for l in (0..codes.len().min(32)).rev() {
        if mask & (1 << l) != 0 {
            let code = codes[l].as_u64().unwrap_or(0) as u16;
            if code != 1 {
                return code;
            }
        }
    }
    0
}
pub fn message(layout: &Value, state: State) -> Value {
    let protocol = layout["protocol"].as_u64().unwrap_or(6) as u32;
    let keys: Vec<Value> = layout["keys"].as_array().into_iter().flatten().map(|key| {
        let code = effective(key["codes"].as_array().map(Vec::as_slice).unwrap_or(&[]),state.mask());
        json!({"label":keycodes::label(code,&layout["customKeycodes"],protocol),"usage":keycodes::usage(code,protocol)})
    }).collect();
    json!({"type":"layers","layer":state.highest(),"layerMask":state.layers,"defaultLayerMask":state.default,"keys":keys})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_reports() {
        let mut b = [0u8; 32];
        b[..6].copy_from_slice(MAGIC);
        b[6] = 0x83;
        b[7] = 1;
        b[8..12].copy_from_slice(&42u32.to_le_bytes());
        b[12] = 4;
        b[16] = 1;
        assert_eq!(
            parse(&b, 42, 10),
            Some(State {
                layers: 4,
                default: 1
            })
        );
        assert_eq!(parse(&b, 41, 10), None);
        assert_eq!(parse(&b, 42, 2), None);
        assert_eq!(parse(&b[..31], 42, 10), None);
        b[5] = 2;
        assert_eq!(parse(&b, 42, 10), None);
    }
    #[test]
    fn respects_active_stack_and_blocked_keys() {
        let codes = vec![json!(4), json!(5), json!(1), json!(0)];
        assert_eq!(effective(&codes, 5), 4);
        assert_eq!(effective(&codes, 7), 5);
        assert_eq!(effective(&codes, 15), 0);
        assert_eq!(
            State {
                layers: 0,
                default: 0
            }
            .highest(),
            0
        );
    }
}
