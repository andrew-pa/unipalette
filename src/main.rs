
use anyhow::*;
use clap::{AppSettings, Clap};
use std::path::PathBuf;

mod palette;
mod preview;
mod expander;

#[derive(Clap)]
enum Commands {
    /// Preview a color palette
    Preview {
        /// Display different lightness variations of each color
        #[clap(long = "shades")]
        show_shades: bool
    },
    /// Process configuration files and expand color expressions
    Expand {
        /// Path to expand. If this is a directory, then all files ending in `.uncol` will be
        /// expanded and the result will be placed besides them without the `.uncol` ending
        path: PathBuf
    },
    /// Evaluate a color expression from the command line
    Eval {
        /// A color expression to evaluate
        expr: String,
        /// Color the output with the result color
        #[clap(short = 'c', long = "colored")]
        colored: bool,
        /// Format for output color:
        /// '#' => sRGB hex,
        /// '~' => Linear sRGB hex,
        /// '$' => CSS RGB decimal,
        /// '~' => CSS LCH decimal
        #[clap(short, default_value="#")]
        output_format: String
    }
}

#[derive(Clap)]
#[clap(about = clap::crate_description!(), author = clap::crate_authors!(), version = clap::crate_version!())]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    /// Specify color palette file to use
    palette: PathBuf,

    #[clap(subcommand)]
    subcmd: Commands
}

fn main() -> Result<()> {
    let opts = Options::parse();

    let source = std::fs::read_to_string(&opts.palette)?;
    let palette = palette::read_palette(&source)?;

    match opts.subcmd {
        Commands::Preview { show_shades } => {
            preview::run(&palette, show_shades, &opts.palette)
        },
        Commands::Eval { expr, colored, output_format } => {
            preview::eval(&palette, expr, colored, output_format)
        },
        Commands::Expand { path } => {
            expander::run(&palette, &path)
        }
    }
}
