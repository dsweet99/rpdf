use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rpdf", version, about = "Local-first PDF parser")]
pub struct Root {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Parse(ParseCli),
    Inspect(InspectCli),
    Render(RenderCli),
}

#[derive(Parser)]
pub struct RenderCli {
    pub input: PathBuf,
    #[arg(long)]
    pub page: u32,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Parser)]
#[allow(clippy::struct_excessive_bools)]
pub struct ParseCli {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<PathBuf>,
    #[arg(long)]
    pub stdout: bool,
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long)]
    pub pages: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long)]
    pub use_struct_tree: bool,
    #[arg(long, value_name = "MODE")]
    pub reading_order: Option<String>,
    #[arg(long, value_name = "MODE")]
    pub table_mode: Option<String>,
    #[arg(long)]
    pub include_header_footer: bool,
    #[arg(long)]
    pub keep_line_breaks: bool,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub debug_json: Option<PathBuf>,
}

#[derive(Parser)]
pub struct InspectCli {
    pub input: PathBuf,
    #[arg(long)]
    pub pages: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
}

#[cfg(test)]
mod kiss_coverage {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn symbols_resolvable() {
        let _ = Root::command;
        let _ = std::mem::size_of::<Root>();
        let _ = std::mem::size_of::<Commands>();
        let _ = std::mem::size_of::<ParseCli>();
        let _ = std::mem::size_of::<InspectCli>();
        let _ = std::mem::size_of::<RenderCli>();
    }
}
