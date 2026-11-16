//! Deterministic real-fixture builders for `iv-media` tests (images, `.cur`, ZIP bombs, ICC/EXIF).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempTestDir {
    pub path: PathBuf,
}

impl TempTestDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("iv_media_test_{prefix}_{}_{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        Self { path: dir }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Builds a synthetic valid `.cur` (`type == 2`) with a PNG payload frame.
pub fn build_synthetic_cur(width: u8, height: u8, png_bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // ICONDIR: reserved=0, type=2 (.CUR), count=1
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    // ICONDIRENTRY (16 bytes)
    out.push(width);
    out.push(height);
    out.push(0); // color count
    out.push(0); // reserved
    out.extend_from_slice(&0u16.to_le_bytes()); // hotspot x
    out.extend_from_slice(&0u16.to_le_bytes()); // hotspot y
    out.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&(22u32).to_le_bytes()); // image offset
    out.extend_from_slice(png_bytes);
    out
}

/// Builds a valid minimal 1x1 or WxH PNG in memory.
pub fn build_rgba_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec()).expect("valid rgba");
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("encode png");
    buf
}

/// Builds a forged EOCD trailer claiming excessive entries or central directory size (`CVE-2026-0429` fixture).
pub fn build_forged_eocd_zip(total_entries: u16, cd_size: u32) -> Vec<u8> {
    let mut eocd = Vec::with_capacity(22);
    eocd.extend_from_slice(&0x06054b50u32.to_le_bytes()); // EOCD sig
    eocd.extend_from_slice(&0u16.to_le_bytes()); // disk
    eocd.extend_from_slice(&0u16.to_le_bytes()); // cd start disk
    eocd.extend_from_slice(&total_entries.to_le_bytes());
    eocd.extend_from_slice(&total_entries.to_le_bytes());
    eocd.extend_from_slice(&cd_size.to_le_bytes());
    eocd.extend_from_slice(&0u32.to_le_bytes()); // cd offset
    eocd.extend_from_slice(&0u16.to_le_bytes()); // comment len
    eocd
}

/// Writes a ZIP file with custom external attributes (e.g., POSIX `0xA000` symlink or traversal name).
pub fn write_zip_with_entries(path: &Path, entries: &[(&str, &[u8], u32)]) {
    let file = std::fs::File::create(path).expect("create zip");
    let mut writer = zip::ZipWriter::new(file);
    for (name, data, ext_attr) in entries {
        if (*ext_attr >> 16) & 0xF000 == 0xA000 {
            writer
                .add_symlink(*name, "target.png", zip::write::SimpleFileOptions::default())
                .expect("add zip symlink");
        } else {
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(*ext_attr >> 16);
            writer.start_file(*name, opts).expect("start zip file");
            writer.write_all(data).expect("write zip data");
        }
    }
    writer.finish().expect("finish zip");
}
