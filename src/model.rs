use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Success,
    PartialSuccess,
    Failure,
}

impl RunStatus {
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::PartialSuccess => 3,
            Self::Failure => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseConfig {
    pub reading_order: String,
    pub table_mode: String,
    pub use_struct_tree: bool,
    pub include_header_footer: bool,
    pub keep_line_breaks: bool,
}

#[derive(Debug, Serialize)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub page: u32,
    pub bbox: [f32; 4],
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Self>,
}

#[derive(Debug, Serialize)]
pub struct PageOut {
    pub page: u32,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<Element>,
}

#[derive(Debug, Serialize)]
pub struct DocumentJson {
    pub schema_version: &'static str,
    pub parser_version: String,
    pub pdfium_binary_tag: &'static str,
    pub status: RunStatus,
    pub input: String,
    pub page_count: u32,
    pub warnings: Vec<String>,
    pub failed_pages: Vec<u32>,
    pub config: ParseConfig,
    pub pages: Vec<PageOut>,
}

pub fn normalize_text(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn model_symbols() {
        let _ = std::mem::size_of::<RunStatus>();
        let _: fn(RunStatus) -> i32 = RunStatus::exit_code;
        let _ = std::mem::size_of::<ParseConfig>();
        let _ = std::mem::size_of::<Element>();
        let _ = std::mem::size_of::<PageOut>();
        let _ = std::mem::size_of::<DocumentJson>();
        let _: fn(&str) -> String = normalize_text;
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn run_status_exit_codes() {
        assert_eq!(RunStatus::Success.exit_code(), 0);
        assert_eq!(RunStatus::PartialSuccess.exit_code(), 3);
        assert_eq!(RunStatus::Failure.exit_code(), 2);
    }

    #[test]
    fn normalize_text_crlf_and_cr() {
        assert_eq!(normalize_text("a\r\nb"), "a\nb");
        assert_eq!(normalize_text("a\rb"), "a\nb");
    }

    #[test]
    fn run_status_serializes_snake_case() {
        let v = serde_json::to_value(RunStatus::PartialSuccess).expect("json");
        assert_eq!(v, serde_json::json!("partial_success"));
    }
}
