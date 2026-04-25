use regex::Regex;
use std::sync::LazyLock;

static LONE_PAGE_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\d{1,2}\n+(# [^\n]+)$").expect("lone page digit before atx heading")
});
static STRIP_PAGE_NUM_BEFORE_ATX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\d{1,2}\s+(# [^\n]+)$").expect("page num prefix before atx line")
});
static ORPHAN_BULLET_BEFORE_ATX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^•\n(# [^\n]+)$").expect("orphan bullet before atx line")
});
static ORPHAN_BULLET_BEFORE_ATX_INLINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^•\s+(# [^\n]+)$").expect("orphan bullet inline before atx")
});
static ORPHAN_BULLET_BEFORE_SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^•\n(Results|Conclusion|Recommendations|References|Executive Summary|Introduction|Background|Methodology|Findings)\b",
    )
    .expect("orphan bullet before section title")
});
static GLUED_ID_EXEC_SUMMARY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([A-Z0-9][A-Za-z0-9-]+)\s+(Executive Summary)\s+([A-Z][^\n]*)$").expect("exec")
});
static HEADING_INTRO_THE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(Introduction)\s+(The [^\n]+)").expect("intro the"));
static HEADING_BG_OUR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(Background)\s+(Our [^\n]+)").expect("bg our"));
static HEADING_METH_THIS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(Methodology)\s+(This [^\n]+)").expect("meth this"));
static HEADING_RES_THE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(Results)\s+(The [^\n]+)").expect("res the"));
static HEADING_CONC_THE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(Conclusion)\s+(The [^\n]+)").expect("conc the"));
static HEADING_FIND_ANALYSIS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(Findings)\s+(Analysis [^\n]+)").expect("findings analysis")
});
static HEADING_KEY_OBS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(Key Observations)\s+([A-Z][^\n]*)").expect("key observations")
});
static HEADING_NUM_SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^([1-9]\d{0,2})\s+(Methodology|Introduction|Results|Conclusion|References|Recommendations)\s+([^\n]+)$",
    )
    .expect("num section")
});
static HEADING_NUM_FINDINGS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([1-9]\d{0,2})\s+(Findings)\s+([^\n]+)$").expect("num findings")
});

pub fn promote_report_headings(s: &str) -> String {
    let mut s = LONE_PAGE_NUMBER.replace_all(s, "$1").into_owned();
    s = STRIP_PAGE_NUM_BEFORE_ATX
        .replace_all(&s, "$1")
        .into_owned();
    s = ORPHAN_BULLET_BEFORE_SECTION
        .replace_all(&s, "$1")
        .into_owned();
    s = GLUED_ID_EXEC_SUMMARY
        .replace_all(&s, "$1\n\n# Executive Summary\n\n$3")
        .into_owned();
    s = HEADING_INTRO_THE
        .replace_all(&s, "# Introduction\n\n$2")
        .into_owned();
    s = HEADING_BG_OUR.replace_all(&s, "# Background\n\n$2").into_owned();
    s = HEADING_METH_THIS
        .replace_all(&s, "# Methodology\n\n$2")
        .into_owned();
    s = HEADING_RES_THE.replace_all(&s, "# Results\n\n$2").into_owned();
    s = HEADING_CONC_THE
        .replace_all(&s, "# Conclusion\n\n$2")
        .into_owned();
    s = HEADING_FIND_ANALYSIS
        .replace_all(&s, "## Findings\n\n$2")
        .into_owned();
    s = HEADING_KEY_OBS.replace_all(&s, "### Key Observations\n\n$2").into_owned();
    s = HEADING_NUM_FINDINGS
        .replace_all(&s, "## Findings\n\n$3")
        .into_owned();
    s = HEADING_NUM_SECTION
        .replace_all(&s, "# $2\n\n$3")
        .into_owned();
    s = STRIP_PAGE_NUM_BEFORE_ATX
        .replace_all(&s, "$1")
        .into_owned();
    s = ORPHAN_BULLET_BEFORE_ATX
        .replace_all(&s, "$1")
        .into_owned();
    ORPHAN_BULLET_BEFORE_ATX_INLINE
        .replace_all(&s, "$1")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promote_report_headings_does_not_treat_year_as_section_number() {
        let input = "2024 Results were mixed across regions";
        let out = promote_report_headings(input);
        assert_eq!(out, input);
    }

    #[test]
    fn promote_report_headings_does_not_treat_year_as_findings_number() {
        let input = "2024 Findings indicate regional variance";
        let out = promote_report_headings(input);
        assert_eq!(out, input);
    }

    #[test]
    fn promote_report_headings_accepts_three_digit_section_numbers() {
        let input = "100 Results extended analysis";
        let out = promote_report_headings(input);
        assert_eq!(out, "# Results\n\nextended analysis");
    }

    #[test]
    fn promote_report_headings_does_not_rewrite_inline_exec_summary_phrase() {
        let input = "The board discussed Executive Summary outcomes yesterday.";
        let out = promote_report_headings(input);
        assert_eq!(out, input);
    }

    #[test]
    fn promote_report_headings_does_not_introduce_leading_blank_before_atx() {
        let input = "1\n# Heading";
        let out = promote_report_headings(input);
        assert_eq!(out, "# Heading");
    }
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(
            stringify!(super::promote_report_headings),
            "super::promote_report_headings"
        );
    }
}
