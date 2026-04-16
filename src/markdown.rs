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
    blocks.join("\n")
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
}
