mod args;
mod editor;
mod theme;
use crate::{args::Args, editor::Editor};

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    let mut editor = Editor::new(args.file_path);
    editor.run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}
