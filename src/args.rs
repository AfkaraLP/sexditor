use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Input File
    // #[arg(short, long)]
    pub file_path: Option<String>,
}
