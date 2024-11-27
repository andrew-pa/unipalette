
use std::{fs::read_dir, path::{ Path, PathBuf }};
use anyhow::{Result, anyhow};
use rayon::prelude::*;
use crate::palette::{Palette, color_parser};
use std::io::Write;
use regex::{Regex, Captures};
use palette::{Clamp, FromColor};

lazy_static::lazy_static! {
    static ref SRC_REGEX: Regex =
        Regex::new(r#"~~!([aA])?([#\$!~])([^!]*)!"#).expect("compile regex");
}

fn find_eligable_under(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in read_dir(path)? {
        let entry = entry?;
        if entry.path().is_dir() {
            find_eligable_under(&entry.path(), files)?;
        } else if entry.file_name().to_str().unwrap().ends_with(".uncol") {
            files.push(entry.path())
        }
    }
    Ok(())
}

pub enum ColorOutputRep {
    Hash(bool), LinHash(bool), CssRgb, CssLch
}

pub fn fmt_color(col: crate::palette::Lcha, output_type: ColorOutputRep, with_alpha: bool) -> String {
    match output_type {
        ColorOutputRep::Hash(alpha_at_start) => {
            let (fcr, fcg, fcb, fca): (u8,u8,u8,u8) = palette::Srgba::from_color(col).clamp().into_format().into_components();
            if with_alpha {
                if alpha_at_start {
                    format!("#{:02x}{:02x}{:02x}{:02x}", fca, fcr, fcg, fcb)
                } else {
                    format!("#{:02x}{:02x}{:02x}{:02x}", fcr, fcg, fcb, fca)
                }
            } else {
                format!("#{:02x}{:02x}{:02x}", fcr, fcg, fcb)
            }
        },
        ColorOutputRep::LinHash(alpha_at_start) => {
            let (fcr, fcg, fcb, fca): (u8,u8,u8,u8) = palette::LinSrgba::from_color(col).clamp().into_format().into_components();
            if with_alpha {
                if alpha_at_start {
                    format!("#{:02x}{:02x}{:02x}{:02x}", fca, fcr, fcg, fcb)
                } else {
                    format!("#{:02x}{:02x}{:02x}{:02x}", fcr, fcg, fcb, fca)
                }
            } else {
                format!("#{:02x}{:02x}{:02x}", fcr, fcg, fcb)
            }
        },
        ColorOutputRep::CssRgb => {
            let (fcr, fcg, fcb, fca): (f32,f32,f32,f32) = palette::Srgba::from_color(col).clamp().into_format().into_components();
            if with_alpha {
                format!("rgba({:.2}%, {:.2}%, {:.2}%, {:.2})", fcr*100.0, fcg*100.0, fcb*100.0, fca)
            } else {
                format!("rgb({:.2}%, {:.2}%, {:.2}%)", fcr*100.0, fcg*100.0, fcb*100.0)
            }
        },
        ColorOutputRep::CssLch => {
            if with_alpha {
                format!("lch({}% {} {} / {})", col.l, col.chroma, col.hue.into_positive_degrees(), col.alpha)
            } else {
                format!("lch({}% {} {})", col.l, col.chroma, col.hue.into_positive_degrees())
            }
        }
    }
}

fn process_file(file: &Path, palette: &Palette) -> Result<()> {
    assert!(file.is_file());
    let new_file_path = file.with_file_name(file.file_stem()
                                            .and_then(|s| s.to_str())
                                            .ok_or(anyhow!("invalid file name {}", file.display()))?);
    println!("processing {} => {}", file.display(), new_file_path.display());
    let mut output = std::fs::File::create(new_file_path)?;
    let input = std::fs::read_to_string(file)?;
    let processed = SRC_REGEX.replace_all(&input, |caps: &Captures| {
        //dbg!(caps);
        let col = match color_parser::color(&caps[3]) {
            Ok(cs) => match cs.resolve(palette) {
                Ok(c) => c,
                Err(e) => {
                    println!("[{}] failed to resolve color spec \"{}\": {}", file.display(), &caps[3], e);
                    return String::new();
                }
            },
            Err(e) => {
                println!("[{}] failed to parse color spec \"{}\": {}", file.display(), &caps[3], e);
                return String::new();
            }
        };
        let alpha_at_start = caps.get(1).map_or(false, |s| s.as_str() == "A");
        fmt_color(col, match &caps[2] {
            "#" => ColorOutputRep::Hash(alpha_at_start),
            "~" => ColorOutputRep::LinHash(alpha_at_start),
            "$" => ColorOutputRep::CssRgb,
            "!" => ColorOutputRep::CssLch,
            _ => unreachable!()
        }, caps.get(1).is_some())
    });
    output.write_all(processed.as_bytes())?;
    Ok(())
}

pub fn run(palette: &Palette, path: &Path) -> Result<()> {
    if path.is_dir() {
        let mut list_of_files = Vec::new();
        find_eligable_under(path, &mut list_of_files)?;
        list_of_files.into_par_iter().for_each(|p| match process_file(&p, palette) {
            Ok(()) => {},
            Err(e) => {
                println!("error processing {}: {}", p.display(), e);
            }
        });
        Ok(())
    } else {
        process_file(path, palette)
    }
}
