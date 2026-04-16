use flate2::read::GzDecoder;
use pdfium_render::prelude::*;
use std::fs::{self, File};
use std::io::copy;
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
    if !tgz.is_file() {
        let url = format!(
            "https://github.com/bblanchon/pdfium-binaries/releases/download/{PDFIUM_BINARY_TAG}/pdfium-linux-x64.tgz"
        );
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| format!("download Pdfium: {e}"))?;
        let mut f = File::create(&tgz).map_err(|e| e.to_string())?;
        copy(&mut resp.into_reader(), &mut f).map_err(|e| e.to_string())?;
    }
    let unpack = base.join("unpack");
    if unpack.exists() {
        fs::remove_dir_all(&unpack).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&unpack).map_err(|e| e.to_string())?;
    let gz = File::open(&tgz).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(GzDecoder::new(gz));
    archive.unpack(&unpack).map_err(|e| format!("unpack Pdfium: {e}"))?;
    let from = unpack.join("lib").join("libpdfium.so");
    if !from.is_file() {
        return Err("libpdfium.so missing after unpack".to_string());
    }
    fs::copy(&from, &lib).map_err(|e| e.to_string())?;
    Ok(lib_dir)
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
}
