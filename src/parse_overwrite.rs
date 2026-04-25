use std::path::Path;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OverwriteTarget {
    Markdown,
    Json,
    Debug,
}

pub fn first_existing_output<'a>(
    markdown: Option<&'a Path>,
    json: Option<&'a Path>,
    debug: Option<&'a Path>,
) -> Option<(OverwriteTarget, &'a Path)> {
    if let Some(path) = markdown.filter(|p| p.exists()) {
        return Some((OverwriteTarget::Markdown, path));
    }
    if let Some(path) = json.filter(|p| p.exists()) {
        return Some((OverwriteTarget::Json, path));
    }
    debug
        .filter(|p| p.exists())
        .map(|path| (OverwriteTarget::Debug, path))
}

pub fn emit_overwrite(path: &Path) {
    eprintln!("refusing to overwrite {}", path.display());
}
