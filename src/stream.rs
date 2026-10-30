use crate::{
    Result,
    core::{self, Keys},
    devices, emit, err, theme,
};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, RawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
};
fn open(path: &str, write: bool) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(write)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?)
}
fn poll(fds: &[RawFd], timeout: i32) -> Result<Vec<usize>> {
    let mut poll: Vec<_> = fds
        .iter()
        .map(|fd| libc::pollfd {
            fd: *fd,
            events: libc::POLLIN,
            revents: 0,
        })
        .collect();
    loop {
        let n = unsafe { libc::poll(poll.as_mut_ptr(), poll.len() as libc::nfds_t, timeout) };
        if n < 0 {
            let e = std::io::Error::last_os_error();
            if e.kind() == std::io::ErrorKind::Interrupted {
                continue;
            };
            return Err(e.into());
        }
        break;
    }
    Ok(poll
        .iter()
        .enumerate()
        .filter(|(_, p)| p.revents != 0)
        .map(|(i, _)| i)
        .collect())
}
pub(crate) fn query(file: &mut File, payload: &[u8]) -> Result<[u8; 32]> {
    if payload.is_empty()
        || payload.len() > 32
        || !matches!(payload[0], 1 | 0x11 | 0x12 | 0xfe)
        || payload[0] == 0xfe && (payload.len() < 2 || payload[1] > 2)
    {
        return err("Not a read-only Vial command");
    }
    let mut packet = [0u8; 33];
    packet[1..payload.len() + 1].copy_from_slice(payload);
    if file.write(&packet)? != 33 {
        return err("Short keyboard write");
    };
    if poll(&[file.as_raw_fd()], 1000)?.is_empty() {
        return err("Keyboard did not answer; close Vial and reconnect");
    };
    let mut out = [0; 32];
    if file.read(&mut out)? != 32 {
        return err("Short keyboard response");
    };
    Ok(out)
}
fn snapshot(path: &str) -> Result<Value> {
    let mut file = open(path, true)?;
    let identity = query(&mut file, &[254, 0])?;
    if u32::from_le_bytes(identity[..4].try_into()?) > 6 {
        return err("Unsupported Vial protocol");
    };
    let response = query(&mut file, &[254, 1])?;
    let size = u32::from_le_bytes(response[..4].try_into()?) as usize;
    if size == 0 || size > 262144 {
        return err("Invalid keyboard definition size");
    };
    let mut compressed = Vec::new();
    for i in 0..size.div_ceil(32) {
        let mut p = vec![254, 2];
        p.extend((i as u32).to_le_bytes());
        compressed.extend(query(&mut file, &p)?)
    }
    compressed.truncate(size);
    let d = core::decode(&compressed)?;
    let layers = query(&mut file, &[17])?[1] as usize;
    let (rows, cols) = core::dimensions(&d)?;
    let total = layers * rows * cols * 2;
    if !(1..=32).contains(&layers) || total > 65535 {
        return err("Invalid matrix dimensions");
    };
    let mut raw = Vec::new();
    for offset in (0..total).step_by(28) {
        let count = (total - offset).min(28);
        let pos = (offset as u16).to_be_bytes();
        let r = query(&mut file, &[18, pos[0], pos[1], count as u8])?;
        raw.extend_from_slice(&r[4..4 + count]);
    }
    let codes: Vec<_> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    let mut layout = core::model(&d, layers, &codes)?;
    layout["source"] = json!("vial");
    Ok(layout)
}
fn key_state(fd: RawFd) -> Result<Keys> {
    let mut bits = [0u8; 96];
    let request = (2u64 << 30) | (96 << 16) | ((b'E' as u64) << 8) | 0x18;
    let result = unsafe { libc::ioctl(fd, request as libc::c_ulong, bits.as_mut_ptr()) };
    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    };
    Ok((0..768)
        .filter(|i| bits[i / 8] & (1 << (i % 8)) != 0)
        .map(|i| i as u16)
        .collect())
}
#[derive(Default)]
pub struct Reader {
    pub held: Keys,
    buffer: Vec<u8>,
    dropped: bool,
}
impl Reader {
    pub fn feed(
        &mut self,
        data: &[u8],
        mut resync: impl FnMut() -> Result<Keys>,
    ) -> Result<Vec<Keys>> {
        self.buffer.extend_from_slice(data);
        let size = std::mem::size_of::<libc::input_event>();
        let offset = std::mem::size_of::<libc::timeval>();
        let count = self.buffer.len() / size;
        let mut states = Vec::new();
        for event in self.buffer[..count * size].chunks_exact(size) {
            let kind = u16::from_ne_bytes(event[offset..offset + 2].try_into()?);
            let code = u16::from_ne_bytes(event[offset + 2..offset + 4].try_into()?);
            let value = i32::from_ne_bytes(event[offset + 4..offset + 8].try_into()?);
            match (kind, code) {
                (0, 3) => {
                    self.dropped = true;
                    self.held.clear();
                    states.push(Keys::new());
                }
                (0, 0) => {
                    if self.dropped {
                        self.held = resync()?;
                        self.dropped = false;
                    }
                    states.push(self.held.clone());
                }
                (1, _) if !self.dropped => {
                    if value == 1 {
                        self.held.insert(code);
                    } else if value == 0 {
                        self.held.remove(&code);
                    }
                }
                _ => (),
            }
        }
        self.buffer.drain(..count * size);
        Ok(states)
    }
}
struct Input {
    file: File,
    kind: String,
    reader: Reader,
    held: Keys,
}
fn control_read(buffer: &mut Vec<u8>) -> Result<bool> {
    let mut bytes = [0u8; 4096];
    let n = unsafe { libc::read(0, bytes.as_mut_ptr().cast(), bytes.len()) };
    if n < 0 {
        return Err(std::io::Error::last_os_error().into());
    };
    if n == 0 {
        return Ok(false);
    }
    buffer.extend_from_slice(&bytes[..n as usize]);
    if buffer.len() > 8192 {
        return err("Oversized control message");
    };
    Ok(true)
}
fn connected(device_id: Option<&str>, backend: &str, preset: &str) -> Result<bool> {
    let device = if let Some(id) = device_id {
        devices::select(id)?
    } else {
        if backend == "standard" {
            return err("Select a keyboard in Settings");
        };
        let supported: Vec<_> = devices::discover()?
            .into_iter()
            .filter(|d| d["vial"] == true)
            .collect();
        if supported.len() != 1 {
            return err("Select a keyboard in Settings");
        };
        supported[0].clone()
    };
    let mut inputs = Vec::new();
    let layout = if backend == "standard" {
        let layout = core::standard(preset)?;
        for node in device["events"].as_array().ok_or("No input interfaces")? {
            let path = crate::string(node)?;
            let file = open(path, false)?;
            let st = file.metadata()?;
            if json!([st.dev(), st.ino(), st.rdev()]) != device["eventIdentity"][path] {
                return err("Keyboard changed during connection; retrying");
            };
            let held = key_state(file.as_raw_fd())?;
            inputs.push(Input {
                file,
                kind: "standard".into(),
                reader: Reader {
                    held: held.clone(),
                    ..Default::default()
                },
                held,
            });
        }
        layout
    } else {
        if device["vial"] != true {
            return err("Select Standard mode for this keyboard");
        };
        let paths = devices::hid(&device)?;
        let layout = snapshot(
            paths
                .get("vial")
                .ok_or("Selected Vial keyboard disconnected")?,
        )?;
        for kind in ["boot", "shared"] {
            if let Some(path) = paths.get(kind) {
                inputs.push(Input {
                    file: open(path, false)?,
                    kind: kind.into(),
                    reader: Reader::default(),
                    held: Keys::new(),
                });
            }
        }
        layout
    };
    if inputs.is_empty() {
        return err("No keyboard input interface available");
    };
    let mut current_theme = theme();
    emit(&json!({"type":"layout","layout":layout,"theme":current_theme}))?;
    let mut previous = Value::Null;
    let mut suggest = true;
    let mut control = Vec::new();
    let (mut keyboard, mut media) = (Keys::new(), Keys::new());
    if backend == "standard" {
        let held: Keys = inputs.iter().flat_map(|i| i.held.iter().copied()).collect();
        previous = json!({"type":"state","pressed":held,"estimate":null});
        emit(&previous)?;
    }
    loop {
        let fds: Vec<_> = std::iter::once(0)
            .chain(inputs.iter().map(|i| i.file.as_raw_fd()))
            .collect();
        let ready = poll(&fds, -1)?;
        let mut changed = false;
        if ready.contains(&0) {
            if !control_read(&mut control)? {
                return Ok(false);
            }
            while let Some(pos) = control.iter().position(|b| *b == b'\n') {
                let command: Vec<_> = control.drain(..=pos).collect();
                match command.as_slice() {
                    b"theme\n" => {
                        let updated = theme();
                        if updated != current_theme {
                            current_theme = updated;
                            emit(&json!({"type":"theme","theme":current_theme}))?;
                        }
                    }
                    b"suggest:0\n" => {
                        suggest = false;
                        changed = true;
                    }
                    b"suggest:1\n" => {
                        suggest = true;
                        changed = true;
                    }
                    _ => (),
                }
            }
        }
        for index in ready.into_iter().filter(|i| *i > 0).map(|i| i - 1) {
            for _ in 0..256 {
                let mut data = [0u8; 1536];
                let size = if backend == "standard" {
                    data.len()
                } else {
                    64
                };
                let n = match inputs[index].file.read(&mut data[..size]) {
                    Ok(0) => return err("Keyboard disconnected"),
                    Ok(n) => n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e.into()),
                };
                if backend == "standard" {
                    let fd = inputs[index].file.as_raw_fd();
                    let states = inputs[index].reader.feed(&data[..n], || key_state(fd))?;
                    for state in states {
                        inputs[index].held = state;
                        let held: Keys =
                            inputs.iter().flat_map(|i| i.held.iter().copied()).collect();
                        let message = json!({"type":"state","pressed":held,"estimate":null});
                        if message != previous {
                            emit(&message)?;
                            previous = message;
                        }
                    }
                } else if let Some((group, codes)) = core::report(&data[..n], &inputs[index].kind) {
                    let held = if group == "media" {
                        &mut media
                    } else {
                        &mut keyboard
                    };
                    if *held != codes {
                        *held = codes;
                        changed = true;
                    }
                }
            }
        }
        if changed && backend != "standard" {
            let held: Keys = keyboard.union(&media).copied().collect();
            let estimate = if suggest {
                core::infer(&layout, &held)
            } else {
                None
            };
            let message = json!({"type":"state","pressed":held,"estimate":estimate});
            if message != previous {
                emit(&message)?;
                previous = message;
            }
        }
    }
}
pub fn main(args: &[String]) -> Result<()> {
    let (mut id, mut backend, mut preset) = (None, "vial", "ansi104");
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let value = iter.next().ok_or("Missing option value")?;
        match arg.as_str() {
            "--device" => id = Some(value.as_str()),
            "--backend" => backend = value,
            "--preset" => preset = value,
            _ => return err("Unknown bridge option"),
        }
    }
    if !matches!(backend, "vial" | "standard") {
        return err("Invalid backend");
    };
    core::standard(preset)?;
    unsafe {
        let flags = libc::fcntl(0, libc::F_GETFL);
        if flags < 0 || libc::fcntl(0, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    loop {
        match connected(id, backend, preset) {
            Ok(false) => return Ok(()),
            Ok(true) => (),
            Err(e) => emit(&json!({"type":"error","message":e.to_string()}))?,
        }
        if !poll(&[0], 2000)?.is_empty() && !control_read(&mut Vec::new())? {
            return Ok(());
        }
    }
}
