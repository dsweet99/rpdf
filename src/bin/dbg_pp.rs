use rpdf::postprocess_extracted_markdown;
fn main() {
    let s = "## Quarterly Revenue | Quarter | Revenue | Growth |\n| --- | --- | --- |\n| Q1 | $2.5M | 15% |";
    let o = postprocess_extracted_markdown(s);
    println!("---OUT---");
    println!("{o}");
    println!("---END---");
}

#[cfg(test)]
mod tests {
    use rpdf::postprocess_extracted_markdown;

    #[test]
    fn sample_debug_fixture_stays_table_like() {
        let s = "## Quarterly Revenue | Quarter | Revenue | Growth |\n| --- | --- | --- |\n| Q1 | $2.5M | 15% |";
        let out = postprocess_extracted_markdown(s);
        assert!(out.contains("## Quarterly Revenue"));
        assert!(out.contains("| --- | --- | --- |"));
    }
}
