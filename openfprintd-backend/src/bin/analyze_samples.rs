use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sil6250::{Features, Frame};

const DW: usize = 64;
const DH: usize = 80;
const DN: usize = DW * DH;

fn main() -> anyhow::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/gxfp-samples".to_owned());
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| {
            let p = e.ok()?.path();
            (p.extension()?.to_str()? == "pgm").then_some(p)
        })
        .collect();
    paths.sort();

    for variant in Variant::all() {
        println!("\n=== {variant:?} ===");
        let mut samples = Vec::new();
        for p in &paths {
            let raw = pgm_to_raw(p, variant)?;
            let frame = Frame::from_raw(&raw);
            let features = Features::extract(&frame);
            println!(
                "{:<12} quality={:.3} kp={}",
                p.file_name().unwrap().to_string_lossy(),
                frame.quality(),
                features.kp.len()
            );
            samples.push((p.clone(), frame, features));
        }

        println!("pairwise score matrix:");
        print!("{:12}", "");
        for (p, _, _) in &samples {
            print!(" {:>8}", short_name(p));
        }
        println!();
        for (p, _, f) in &samples {
            print!("{:12}", short_name(p));
            for (_, _, g) in &samples {
                print!(" {:8}", f.verify_against(std::slice::from_ref(g)));
            }
            println!();
        }

        let gallery: Vec<Features> = samples
            .iter()
            .filter(|(p, _, _)| short_name(p).starts_with("index"))
            .map(|(_, _, f)| clone_features(f))
            .collect();
        println!("against index gallery:");
        for (p, _, f) in &samples {
            println!("{:<12} {}", short_name(p), f.verify_against(&gallery));
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Variant {
    Transpose,
    RotateCw,
    RotateCcw,
    TransposeFlipBoth,
    CropCenter,
    DownsampleWidthPad,
}

impl Variant {
    fn all() -> &'static [Variant] {
        &[
            Variant::Transpose,
            Variant::RotateCw,
            Variant::RotateCcw,
            Variant::TransposeFlipBoth,
            Variant::CropCenter,
            Variant::DownsampleWidthPad,
        ]
    }
}

fn short_name(p: &Path) -> String {
    p.file_name().unwrap().to_string_lossy().replace(".pgm", "")
}

fn clone_features(f: &Features) -> Features {
    Features::deserialize(&f.serialize()).unwrap()
}

fn pgm_to_raw(path: &Path, variant: &Variant) -> io::Result<Box<[u8; DN]>> {
    let bytes = fs::read(path)?;
    let (w, h, maxval, pos) = parse_pgm_header(&bytes)?;
    if w != 80 || h != 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected 80x64 PGM, got {w}x{h}"),
        ));
    }
    let bpp = if maxval > 255 { 2 } else { 1 };
    let data = &bytes[pos..];
    if data.len() < w * h * bpp {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated PGM",
        ));
    }

    let mut out = Box::new([0u8; DN]);
    for y in 0..DH {
        for x in 0..DW {
            let (sx, sy) = match variant {
                Variant::Transpose => (y, x),
                Variant::RotateCw => (y, DW - 1 - x),
                Variant::RotateCcw => (h - 1 - y.min(h - 1), x),
                Variant::TransposeFlipBoth => (w - 1 - y, h - 1 - x),
                Variant::CropCenter => ((x + 8).min(w - 1), y.min(h - 1)),
                Variant::DownsampleWidthPad => (((x * w) / DW).min(w - 1), y.min(h - 1)),
            };
            out[y * DW + x] = read_scaled(data, w, sx, sy, bpp, maxval);
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
    let w = parse_usize(next_token(buf, &mut pos)?)?;
    let h = parse_usize(next_token(buf, &mut pos)?)?;
    let maxval = parse_usize(next_token(buf, &mut pos)?)?;
    while pos < buf.len() && buf[pos].is_ascii_whitespace() {
        pos += 1;
    }
    Ok((w, h, maxval, pos))
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
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid integer"))
}

fn read_scaled(data: &[u8], width: usize, sx: usize, sy: usize, bpp: usize, maxval: usize) -> u8 {
    let idx = (sy * width + sx) * bpp;
    let v = if bpp == 1 {
        data[idx] as usize
    } else {
        ((data[idx] as usize) << 8) | data[idx + 1] as usize
    };
    ((v * 255 + maxval / 2) / maxval).min(255) as u8
}
