use crate::model::PageOut;

pub fn pages_to_markdown(pages: &[PageOut]) -> String {
    let mut blocks = Vec::new();
    for p in pages {
        for e in &p.elements {
            if e.kind == "paragraph" && e.children.is_empty() {
                blocks.push(e.text.clone());
            }
        }
    }
    blocks.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Element;

    #[test]
    fn markdown_joins_paragraphs() {
        let p = PageOut {
            page: 1,
            width: 100.0,
            height: 100.0,
            elements: vec![Element {
                id: "p1-e1".to_string(),
                kind: "paragraph".to_string(),
                page: 1,
                bbox: [0.0, 0.0, 1.0, 1.0],
                text: "a".to_string(),
                children: Vec::new(),
            }],
        };
        assert_eq!(pages_to_markdown(&[p]), "a");
    }

    #[test]
    fn markdown_joins_pages_with_blank_line() {
        let mk = |page: u32, text: &str| PageOut {
            page,
            width: 100.0,
            height: 100.0,
            elements: vec![Element {
                id: format!("p{page}-e1"),
                kind: "paragraph".to_string(),
                page,
                bbox: [0.0, 0.0, 1.0, 1.0],
                text: text.to_string(),
                children: Vec::new(),
            }],
        };
        let a = mk(1, "a");
        let b = mk(2, "b");
        assert_eq!(pages_to_markdown(&[a, b]), "a\n\nb");
    }
}
