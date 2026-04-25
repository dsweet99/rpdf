use flate2::read::GzDecoder;
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{Read, copy};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const PDFIUM_BINARY_TAG: &str = "chromium/7543";

static PDFIUM_FALLBACK_LIB_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

fn try_bind_pdfium(dir: &Path) -> Option<Pdfium> {
    let p = Pdfium::pdfium_platform_library_name_at_path(dir);
    Pdfium::bind_to_library(p).ok().map(Pdfium::new)
}

#[allow(clippy::default_constructed_unit_structs)]
pub fn init_pdfium() -> Pdfium {
    if std::env::var("RPDF_PDFIUM_LIB_DIR").is_ok() {
        if let Some(dir) = resolve_pdfium_dir_from_env() {
            if let Some(pdfium) = try_bind_pdfium(&dir) {
                return pdfium;
            }
            eprintln!(
                "warning: RPDF_PDFIUM_LIB_DIR is set but loading Pdfium from that path failed; using fallback"
            );
        } else {
            eprintln!(
                "warning: RPDF_PDFIUM_LIB_DIR is set but does not point at a usable libpdfium.so; using fallback"
            );
        }
    }
    if let Some(dir) = pdfium_fallback_lib_dir_cached() {
        if let Some(pdfium) = try_bind_pdfium(&dir) {
            return pdfium;
        }
    }
    Pdfium::default()
}

fn pdfium_fallback_lib_dir_cached() -> Option<PathBuf> {
    PDFIUM_FALLBACK_LIB_DIR
        .get_or_init(resolve_pdfium_lib_dir_fallback)
        .clone()
}

fn resolve_pdfium_dir_from_env() -> Option<PathBuf> {
    let v = std::env::var("RPDF_PDFIUM_LIB_DIR").ok()?;
    if v.is_empty() {
        return None;
    }
    let p = PathBuf::from(v);
    if p.is_dir() && p.join("libpdfium.so").is_file() {
        return Some(p);
    }
    if p.file_name().is_some_and(|n| n == "libpdfium.so") {
        return p.parent().map(std::path::Path::to_path_buf);
    }
    None
}

fn resolve_pdfium_lib_dir_fallback() -> Option<PathBuf> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        fetch_linux_gnu_pdfium().ok()
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        None
    }
}

fn fetch_linux_gnu_pdfium() -> Result<PathBuf, String> {
    let base = dirs::cache_dir()
        .ok_or_else(|| "no user cache directory for Pdfium download".to_string())?
        .join("rpdf")
        .join("pdfium-7543");
    let lib_dir = base.join("lib");
    let lib = lib_dir.join("libpdfium.so");
    if lib.is_file() {
        return Ok(lib_dir);
    }
    fs::create_dir_all(&lib_dir).map_err(|e| e.to_string())?;
    let tgz = base.join("pdfium-linux-x64.tgz");
    ensure_tgz_valid(&tgz)?;
    let from = unpack_pdfium(&base, &tgz)?;
    fs::copy(&from, &lib).map_err(|e| e.to_string())?;
    Ok(lib_dir)
}

fn ensure_tgz_valid(tgz: &Path) -> Result<(), String> {
    ensure_tgz_valid_with(tgz, ensure_tgz_downloaded, verify_tarball_checksum)
}

fn ensure_tgz_valid_with(
    tgz: &Path,
    mut download: impl FnMut(&Path) -> Result<(), String>,
    mut verify: impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let existed_before = tgz.is_file();
    download(tgz)?;
    match verify(tgz) {
        Ok(()) => return Ok(()),
        Err(e)
            if existed_before
                && (e.contains("download Pdfium checksum")
                    || e.contains("read Pdfium checksum")) =>
        {
            return Err(e);
        }
        Err(_) => {}
    }
    let _ = fs::remove_file(tgz);
    download(tgz)?;
    verify(tgz)
}

fn ensure_tgz_downloaded(tgz: &Path) -> Result<(), String> {
    if tgz.is_file() {
        return Ok(());
    }
    let url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/{PDFIUM_BINARY_TAG}/pdfium-linux-x64.tgz"
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("download Pdfium: {e}"))?;
    let mut f = File::create(tgz).map_err(|e| e.to_string())?;
    copy(&mut resp.into_reader(), &mut f).map_err(|e| e.to_string())?;
    Ok(())
}

fn unpack_pdfium(base: &Path, tgz: &Path) -> Result<PathBuf, String> {
    let unpack = base.join("unpack");
    if unpack.exists() {
        fs::remove_dir_all(&unpack).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&unpack).map_err(|e| e.to_string())?;
    let gz = File::open(tgz).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(GzDecoder::new(gz));
    archive.unpack(&unpack).map_err(|e| format!("unpack Pdfium: {e}"))?;
    let from = unpack.join("lib").join("libpdfium.so");
    if from.is_file() {
        Ok(from)
    } else {
        Err("libpdfium.so missing after unpack".to_string())
    }
}

fn verify_tarball_checksum(tgz: &Path) -> Result<(), String> {
    let checksum_url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/{PDFIUM_BINARY_TAG}/pdfium-linux-x64.tgz.sha256"
    );
    let checksum_resp = ureq::get(&checksum_url)
        .call()
        .map_err(|e| format!("download Pdfium checksum: {e}"))?;
    let mut checksum_text = String::new();
    checksum_resp
        .into_reader()
        .read_to_string(&mut checksum_text)
        .map_err(|e| format!("read Pdfium checksum: {e}"))?;
    let expected = parse_checksum_line(&checksum_text)?;
    let data = fs::read(tgz).map_err(|e| format!("read Pdfium archive: {e}"))?;
    let actual = Sha256::digest(data).iter().fold(
        String::with_capacity(64),
        |mut acc, b| {
            write!(&mut acc, "{b:02x}").expect("hex write");
            acc
        },
    );
    if actual == expected {
        Ok(())
    } else {
        Err("Pdfium archive checksum mismatch".to_string())
    }
}

fn parse_checksum_line(raw: &str) -> Result<String, String> {
    let token = raw
        .split_whitespace()
        .next()
        .ok_or_else(|| "empty Pdfium checksum response".to_string())?;
    if token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(token.to_ascii_lowercase())
    } else {
        Err("invalid Pdfium checksum response".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdfium_fallback_lib_dir_cached_symbol() {
        let _: fn() -> Option<PathBuf> = pdfium_fallback_lib_dir_cached;
    }

    #[test]
    fn resolve_pdfium_lib_dir_fallback_symbol() {
        let _: fn() -> Option<PathBuf> = resolve_pdfium_lib_dir_fallback;
    }

    #[test]
    fn init_pdfium_symbol() {
        let _: fn() -> Pdfium = init_pdfium;
    }

    #[test]
    fn symbol_refs() {
        assert_eq!(stringify!(try_bind_pdfium), "try_bind_pdfium");
        assert_eq!(
            stringify!(resolve_pdfium_dir_from_env),
            "resolve_pdfium_dir_from_env"
        );
        assert_eq!(stringify!(fetch_linux_gnu_pdfium), "fetch_linux_gnu_pdfium");
    }

    #[test]
    fn ensure_tgz_valid_with_redownloads_after_failed_verify() {
        let base = std::env::temp_dir().join(format!("rpdf_engine_tgz_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("mkdir");
        let tgz = base.join("pdfium-linux-x64.tgz");
        let mut downloads = 0u8;
        let mut verifies = 0u8;
        let out = ensure_tgz_valid_with(
            &tgz,
            |path| {
                downloads = downloads.saturating_add(1);
                fs::write(path, b"payload").map_err(|e| e.to_string())
            },
            |_path| {
                verifies = verifies.saturating_add(1);
                if verifies == 1 {
                    Err("bad checksum".to_string())
                } else {
                    Ok(())
                }
            },
        );
        assert!(out.is_ok(), "{out:?}");
        assert_eq!(downloads, 2);
        assert_eq!(verifies, 2);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn ensure_tgz_valid_with_keeps_cached_archive_on_checksum_fetch_error() {
        let base = std::env::temp_dir().join(format!("rpdf_engine_cached_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("mkdir");
        let tgz = base.join("pdfium-linux-x64.tgz");
        fs::write(&tgz, b"cached").expect("seed");
        let mut downloads = 0u8;
        let out = ensure_tgz_valid_with(
            &tgz,
            |_path| {
                downloads = downloads.saturating_add(1);
                Ok(())
            },
            |_path| Err("download Pdfium checksum: offline".to_string()),
        );
        assert!(out.is_err());
        assert_eq!(downloads, 1);
        assert!(tgz.is_file());
        let _ = fs::remove_dir_all(&base);
    }
}
