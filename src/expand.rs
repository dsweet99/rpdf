use std::path::{Path, PathBuf};

fn is_pdf(path: &Path) -> bool {
    path.extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("pdf"))
}

fn push_pdfs_under_dir(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let walker = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .sort_by_file_name();
    for entry in walker {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let pb = entry.path().to_path_buf();
        if !is_pdf(&pb) {
            continue;
        }
        out.push(pb);
    }
    Ok(())
}

pub fn expand_inputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            push_pdfs_under_dir(p, &mut out)?;
        } else if p.is_file() {
            out.push(p.clone());
        } else {
            return Err(format!("not a file or directory: {}", p.display()));
        }
    }
    out.sort();
    out.dedup();
    if out.is_empty() {
        return Err("no PDF inputs matched".to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn expand_inputs_finds_pdf_in_dir() {
        let dir = std::env::temp_dir().join(format!("rpdf_expand_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("mkdir");
        let pdf = dir.join("a.pdf");
        fs::write(&pdf, b"%PDF-1.4\n%%EOF\n").expect("write");
        let got = expand_inputs(std::slice::from_ref(&dir)).expect("expand");
        assert_eq!(got, vec![pdf]);
        let _ = fs::remove_dir_all(&dir);
    }
}
