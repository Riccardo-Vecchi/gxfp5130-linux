use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sil6250::{Features, Frame};

const DW: usize = 64;
const DH: usize = 80;
const DN: usize = DW * DH;
const STORAGE_DIR: &str = "/var/lib/open-fprintd/gxfp";
const QUALITY_MIN: f32 = 0.52;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let username = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: enroll_from_pgms <username> <finger> <pgm>..."))?;
    let finger = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: enroll_from_pgms <username> <finger> <pgm>..."))?;
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    if paths.is_empty() {
        anyhow::bail!("need at least one PGM");
    }

    let mut features = Vec::new();
    for path in paths {
        let raw = pgm_to_sil_raw(&path)?;
        let frame = Frame::from_raw(&raw);
        let quality = frame.quality();
        let feat = Features::extract(&frame);
        println!(
            "{} quality={quality:.3} kp={}",
            path.display(),
            feat.kp.len()
        );
        if quality < QUALITY_MIN {
            anyhow::bail!(
                "{} below quality threshold {QUALITY_MIN:.2}",
                path.display()
            );
        }
        features.push(feat);
    }

    save_features(&username, &finger, &features)?;
    println!(
        "saved {} feature sets for {username}/{finger}",
        features.len()
    );
    Ok(())
}

fn save_features(username: &str, finger: &str, features: &[Features]) -> io::Result<()> {
    check_component(username)?;
    check_component(finger)?;

    let path = Path::new(STORAGE_DIR)
        .join(username)
        .join(finger)
        .join("features.bin");
    let parent = path.parent().unwrap();
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    let mut perms = fs::metadata(parent)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(parent, perms)?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    for feat in features {
        let bytes = feat.serialize();
        file.write_all(&(bytes.len() as u32).to_le_bytes())?;
        file.write_all(&bytes)?;
    }
    Ok(())
}

fn check_component(name: &str) -> io::Result<()> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid path component: {name:?}"),
        ));
    }
    Ok(())
}

fn pgm_to_sil_raw(path: &Path) -> io::Result<Box<[u8; DN]>> {
    let bytes = fs::read(path)?;
    let (width, height, maxval, pos) = parse_pgm_header(&bytes)?;
    if width != 80 || height != 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected 80x64 PGM, got {width}x{height}"),
        ));
    }

    let bytes_per_px = if maxval > 255 { 2 } else { 1 };
    let data = &bytes[pos..];
    if data.len() < width * height * bytes_per_px {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated PGM data",
        ));
    }

    let mut out = Box::new([0u8; DN]);
    for y in 0..DH {
        for x in 0..DW {
            out[y * DW + x] = read_scaled(data, width, y, x, bytes_per_px, maxval);
        }
    }
    Ok(out)
}

fn parse_pgm_header(buf: &[u8]) -> io::Result<(usize, usize, usize, usize)> {
    let mut pos = 0usize;
    let magic = next_token(buf, &mut pos)?;
    if magic != b"P5" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not P5 PGM"));
    }
    let width = parse_usize(next_token(buf, &mut pos)?)?;
    let height = parse_usize(next_token(buf, &mut pos)?)?;
    let maxval = parse_usize(next_token(buf, &mut pos)?)?;
    while pos < buf.len() && buf[pos].is_ascii_whitespace() {
        pos += 1;
    }
    Ok((width, height, maxval, pos))
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
    src_x: usize,
    src_y: usize,
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
