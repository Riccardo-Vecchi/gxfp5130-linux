use std::fs;
use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::Duration;

pub use sil6250::{Features, Frame};

const PM_W: usize = 64;
const PM_H: usize = 80;
const PM_N: usize = PM_W * PM_H;

static CAPTURE_ID: AtomicU64 = AtomicU64::new(1);

pub struct Engine {
    psk_path: String,
    helper_path: String,
}

impl Engine {
    pub fn open(psk_path: &str) -> io::Result<Self> {
        let helper_path = std::env::var("GXFP_CAPTURE_HELPER")
            .unwrap_or_else(|_| "/usr/local/bin/gxfp_capture_once".to_owned());
        Ok(Self {
            psk_path: psk_path.to_owned(),
            helper_path,
        })
    }

    pub fn capture(
        &mut self,
        finger_ms: u32,
        quality_min: f32,
        max_retry: u32,
    ) -> Option<(Frame, Box<[u8; PM_N]>)> {
        for attempt in 0..=max_retry {
            let raw = match self.capture_once(finger_ms) {
                Ok(raw) => raw,
                Err(e) => {
                    tracing::debug!(error = %e, "gxfp capture helper failed");
                    sleep(Duration::from_millis(500));
                    return None;
                }
            };
            let frame = Frame::from_raw(&raw);
            let q = frame.quality();
            let ok = q >= quality_min || attempt >= max_retry;
            tracing::debug!(attempt, q, quality_min, ok, "gxfp capture quality");
            if ok {
                return Some((frame, raw));
            }
            self.wait_finger_up(4000);
        }
        None
    }

    pub fn wait_finger_up(&mut self, _timeout_ms: u32) -> bool {
        true
    }

    fn capture_once(&self, timeout_ms: u32) -> io::Result<Box<[u8; PM_N]>> {
        let id = CAPTURE_ID.fetch_add(1, Ordering::Relaxed);
        let out = format!(
            "/tmp/gxfp-openfprintd-capture-{}-{id}.pgm",
            std::process::id()
        );
        let output = Command::new(&self.helper_path)
            .arg("--psk-raw32")
            .arg(&self.psk_path)
            .arg("--out")
            .arg(&out)
            .arg("--timeout-ms")
            .arg(timeout_ms.to_string())
            .output()?;

        if !output.status.success() {
            let _ = fs::remove_file(&out);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = stderr
                .lines()
                .chain(stdout.lines())
                .next()
                .unwrap_or("no helper output");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "{} exited with {}: {detail}",
                    self.helper_path, output.status
                ),
            ));
        }

        let pgm = fs::read(&out)?;
        if let Ok(dir) = std::env::var("GXFP_DEBUG_CAPTURE_DIR") {
            let _ = fs::create_dir_all(&dir);
            let debug_out = format!("{dir}/capture-{}-{id}.pgm", std::process::id());
            let _ = fs::write(debug_out, &pgm);
        }
        let _ = fs::remove_file(&out);
        pgm_to_sil_raw(&pgm)
    }
}

fn pgm_to_sil_raw(pgm: &[u8]) -> io::Result<Box<[u8; PM_N]>> {
    let mut pos = 0usize;
    let magic = next_token(pgm, &mut pos)?;
    if magic != b"P5" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a raw PGM"));
    }

    let width = parse_usize(next_token(pgm, &mut pos)?)?;
    let height = parse_usize(next_token(pgm, &mut pos)?)?;
    let maxval = parse_usize(next_token(pgm, &mut pos)?)?;
    if maxval == 0 || maxval > 65535 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid PGM maxval",
        ));
    }
    if pos >= pgm.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "missing PGM data",
        ));
    }
    while pos < pgm.len() && pgm[pos].is_ascii_whitespace() {
        pos += 1;
    }

    let bytes_per_px = if maxval > 255 { 2 } else { 1 };
    let needed = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(bytes_per_px))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "PGM dimensions overflow"))?;
    if pgm.len() - pos < needed {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated PGM data",
        ));
    }

    let data = &pgm[pos..pos + needed];
    let mut out = Box::new([0u8; PM_N]);

    match (width, height) {
        (80, 64) => {
            for y in 0..PM_H {
                for x in 0..PM_W {
                    out[y * PM_W + x] = read_scaled(data, width, x, y, bytes_per_px, maxval);
                }
            }
        }
        (64, 80) => {
            for y in 0..PM_H {
                for x in 0..PM_W {
                    out[y * PM_W + x] = read_scaled(data, width, y, x, bytes_per_px, maxval);
                }
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported PGM dimensions {width}x{height}"),
            ));
        }
    }

    Ok(out)
}

fn next_token<'a>(buf: &'a [u8], pos: &mut usize) -> io::Result<&'a [u8]> {
    loop {
        while *pos < buf.len() && buf[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        if *pos < buf.len() && buf[*pos] == b'#' {
            while *pos < buf.len() && buf[*pos] != b'\n' {
                *pos += 1;
            }
            continue;
        }
        break;
    }

    let start = *pos;
    while *pos < buf.len() && !buf[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    if start == *pos {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "missing PGM token",
        ));
    }
    Ok(&buf[start..*pos])
}

fn parse_usize(tok: &[u8]) -> io::Result<usize> {
    std::str::from_utf8(tok)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid PGM integer"))
}

fn read_scaled(
    data: &[u8],
    width: usize,
    src_y: usize,
    src_x: usize,
    bytes_per_px: usize,
    maxval: usize,
) -> u8 {
    let idx = (src_y * width + src_x) * bytes_per_px;
    let v = if bytes_per_px == 1 {
        data[idx] as usize
    } else {
        ((data[idx] as usize) << 8) | data[idx + 1] as usize
    };
    ((v * 255 + maxval / 2) / maxval).min(255) as u8
}
