
use anyhow::*;
use clap::{AppSettings, Clap};
use std::path::PathBuf;

mod palette;
mod preview;
mod expander;

#[derive(Clap)]
enum Commands {
    Preview {
        // Display different lightness variations of each color
        #[clap(long = "shades")]
        show_shades: bool
    },
    Expand {
        // Path to expand. If this is a directory, then all files ending in `.uncol` will be
        // expanded and the result will be placed besides them
        path: PathBuf
    }
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    /// Specify palette file to use
    palette: PathBuf,

    #[clap(subcommand)]
    subcmd: Commands
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let palette = palette::load_palette(&opts.palette)?;

    match opts.subcmd {
        Commands::Preview { show_shades } => {
            preview::run(&palette, show_shades, &opts.palette)
        },
        Commands::Expand { path } => {
            expander::run(&palette, &path)
        }
    }
}
