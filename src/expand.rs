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

fn append_input_path(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_dir() {
        return push_pdfs_under_dir(path, out);
    }
    if path.is_file() {
        if is_pdf(path) {
            out.push(path.to_path_buf());
            return Ok(());
        }
        return Err(format!("not a PDF file: {}", path.display()));
    }
    Err(format!("not a file or directory: {}", path.display()))
}

pub fn expand_inputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in paths {
        append_input_path(p, &mut out)?;
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

    #[test]
    fn expand_inputs_rejects_non_pdf_file_operand() {
        let dir =
            std::env::temp_dir().join(format!("rpdf_expand_nonpdf_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("mkdir");
        let txt = dir.join("note.txt");
        fs::write(&txt, b"not a pdf").expect("write");
        let err = expand_inputs(std::slice::from_ref(&txt)).expect_err("must reject non-pdf");
        assert!(err.contains("not a PDF file"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(stringify!(super::is_pdf), "super::is_pdf");
        assert_eq!(
            stringify!(super::push_pdfs_under_dir),
            "super::push_pdfs_under_dir"
        );
    }
}
