use crate::cli::RenderCli;

pub fn run_render(_cli: &RenderCli) -> i32 {
    eprintln!("rpdf render is not implemented yet");
    1
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;

    #[test]
    fn render_symbol() {
        let _: fn(&RenderCli) -> i32 = run_render;
    }
}
