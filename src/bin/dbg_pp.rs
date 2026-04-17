use rpdf::postprocess_extracted_markdown;
fn main() {
    let s = "## Quarterly Revenue | Quarter | Revenue | Growth |\n| --- | --- | --- |\n| Q1 | $2.5M | 15% |";
    let o = postprocess_extracted_markdown(s);
    println!("---OUT---");
    println!("{o}");
    println!("---END---");
}
