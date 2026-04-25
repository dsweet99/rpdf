const MAX_EXPANDED_PAGE_RANGE: u32 = 100_000;

pub fn parse_pageset(spec: &str) -> Result<std::collections::BTreeSet<u32>, String> {
    let mut out = std::collections::BTreeSet::new();
    for raw in spec.split(',') {
        let p = raw.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((a, b)) = p.split_once('-') {
            let start: u32 = a
                .trim()
                .parse()
                .map_err(|_| format!("invalid page range fragment {p:?}"))?;
            let end: u32 = b
                .trim()
                .parse()
                .map_err(|_| format!("invalid page range fragment {p:?}"))?;
            if start == 0 || end == 0 {
                return Err("page numbers must be >= 1".to_string());
            }
            if start > end {
                return Err(format!("invalid page range {p:?}"));
            }
            if end - start + 1 > MAX_EXPANDED_PAGE_RANGE {
                return Err(format!("page range too large {p:?}"));
            }
            for n in start..=end {
                out.insert(n);
            }
        } else {
            let n: u32 = p
                .parse()
                .map_err(|_| format!("invalid page token {p:?}"))?;
            if n == 0 {
                return Err("page numbers must be >= 1".to_string());
            }
            out.insert(n);
        }
    }
    if out.is_empty() {
        return Err("empty --pages specification".to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pageset_parses_list_and_ranges() {
        let s = parse_pageset("1,3,5-7").expect("ok");
        assert!(s.contains(&1));
        assert!(s.contains(&3));
        assert!(s.contains(&5));
        assert!(s.contains(&6));
        assert!(s.contains(&7));
    }

    #[test]
    fn parse_pageset_rejects_descending_range() {
        assert!(parse_pageset("3-1").is_err());
    }

    #[test]
    fn parse_pageset_rejects_excessive_range_size() {
        let err = parse_pageset("1-100001").expect_err("must reject huge range");
        assert!(err.contains("page range too large"), "{err}");
    }
}
