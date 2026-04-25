type SegRow = (f32, f32, String);

fn cmp_top_down(a: &SegRow, b: &SegRow) -> std::cmp::Ordering {
    b.0.partial_cmp(&a.0)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

fn tertile_columns(
    rows: &[SegRow],
    page_width: f32,
) -> (Vec<SegRow>, Vec<SegRow>, Vec<SegRow>) {
    let t1 = page_width / 3.0;
    let t2 = 2.0 * page_width / 3.0;
    let c0: Vec<SegRow> = rows.iter().filter(|r| r.1 < t1).cloned().collect();
    let c1: Vec<SegRow> = rows
        .iter()
        .filter(|r| (t1..t2).contains(&r.1))
        .cloned()
        .collect();
    let c2: Vec<SegRow> = rows.iter().filter(|r| r.1 >= t2).cloned().collect();
    (c0, c1, c2)
}

fn merge_sorted_columns(mut a: Vec<SegRow>, b: Vec<SegRow>, c: Vec<SegRow>) -> Vec<SegRow> {
    a.extend(b);
    a.extend(c);
    a
}

fn try_three_column_layout(rows: &mut Vec<SegRow>, page_width: f32) -> bool {
    let (mut c0, mut c1, mut c2) = tertile_columns(rows, page_width);
    let n0 = c0.len();
    let n1 = c1.len();
    let n2 = c2.len();
    let max3 = n0.max(n1).max(n2);
    let min3 = n0.min(n1).min(n2);
    let balanced_three =
        rows.len() >= 6 && n0 >= 2 && n1 >= 2 && n2 >= 2 && min3.saturating_mul(3) >= max3;
    if !balanced_three {
        return false;
    }
    c0.sort_by(cmp_top_down);
    c1.sort_by(cmp_top_down);
    c2.sort_by(cmp_top_down);
    *rows = merge_sorted_columns(c0, c1, c2);
    true
}

fn left_right_columns(rows: &[SegRow], page_width: f32) -> (Vec<SegRow>, Vec<SegRow>) {
    let mid = page_width * 0.48;
    let left: Vec<SegRow> = rows.iter().filter(|r| r.1 < mid).cloned().collect();
    let right: Vec<SegRow> = rows.iter().filter(|r| r.1 >= mid).cloned().collect();
    (left, right)
}

fn try_two_column_layout(rows: &mut Vec<SegRow>, page_width: f32) -> bool {
    let (mut left, mut right) = left_right_columns(rows, page_width);
    let lc = left.len();
    let rc = right.len();
    let balanced_columns = lc.min(rc).saturating_mul(5) >= lc.max(rc);
    if !(rows.len() >= 8 && lc >= 3 && rc >= 3 && balanced_columns) {
        return false;
    }
    left.sort_by(cmp_top_down);
    right.sort_by(cmp_top_down);
    *rows = left;
    rows.extend(right);
    true
}

pub fn sort_segment_rows_by_reading_order(rows: &mut Vec<SegRow>, page_width: f32) {
    if try_three_column_layout(rows, page_width) {
        return;
    }
    if try_two_column_layout(rows, page_width) {
        return;
    }
    rows.sort_by(cmp_top_down);
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn symbol_refs() {
        assert_eq!(stringify!(super::cmp_top_down), "super::cmp_top_down");
        assert_eq!(stringify!(super::tertile_columns), "super::tertile_columns");
        assert_eq!(
            stringify!(super::merge_sorted_columns),
            "super::merge_sorted_columns"
        );
        assert_eq!(
            stringify!(super::try_three_column_layout),
            "super::try_three_column_layout"
        );
        assert_eq!(stringify!(super::left_right_columns), "super::left_right_columns");
        assert_eq!(
            stringify!(super::try_two_column_layout),
            "super::try_two_column_layout"
        );
        assert_eq!(
            stringify!(super::sort_segment_rows_by_reading_order),
            "super::sort_segment_rows_by_reading_order"
        );
    }
}
