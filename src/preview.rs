
use anyhow::{Result, bail};
use crate::palette::Palette;
use std::io::Write;
use crossterm::{queue, style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor}};
use palette::{Clamp, FromColor, Lighten};

pub fn run(palette: &Palette, show_shades: bool, path: &std::path::Path) -> Result<()> {
    let mut stdout = std::io::stdout();

    queue!(stdout, SetAttribute(Attribute::Bold),
        Print("Unipalette"),
        SetAttribute(Attribute::Reset),
        Print("|"),
        Print(path.display()),
        Print("\n"),
        )?;

    let mut colors: Vec<_> = palette.colors.iter().collect();
    // we need a way to keep dark colors and light colors seperate as well
    colors.sort_by(|(n1,c1), (n2, c2)| match (c1.l*c1.hue.into_raw_degrees()).partial_cmp(&(c2.hue.into_raw_degrees()*c2.l)) {
        None | Some(std::cmp::Ordering::Equal) => { n1.cmp(n2) },
        o => o.unwrap()
    });

    if show_shades {
        let shades = [-0.5, -0.25, 0.0, 0.25, 0.5];
        let max_name_len = palette.colors.keys().map(|s| s.len()).max().unwrap_or(0);
        queue!(stdout, SetAttribute(Attribute::Underlined))?;
        for _ in 0..(max_name_len+2) {
            queue!(stdout, Print(" "))?;
        }
        for sh in shades {
            write!(stdout, "{:^5}", sh)?;
        }
        queue!(stdout, Print("\n"), SetAttribute(Attribute::Reset))?;


        for (name, color) in colors {
            queue!(stdout, Print(name))?;
            for _ in 0..(max_name_len - name.len()+2) {
                queue!(stdout, Print(" "))?;
            }
            for sh in shades {
                let col = palette::Srgb::from_color(color.lighten_fixed(sh)).into_format().into_components();
                queue!(stdout, SetForegroundColor(Color::Rgb {
                    r: col.0, g: col.1, b: col.2
                }), Print("█████"))?;
            }
            queue!(stdout, ResetColor, Print("\n"))?;
        }
    } else {
        let width = crossterm::terminal::size()?.0;
        let mut cur_width = 0;
        for (name, color) in colors {
            let tx = if name.len() > 16 {
                format!("{}…", &name[0..15])
            } else {
                format!("{:^16}", name)
            };

            let col: (u8,u8,u8,u8) = palette::Srgba::from_color(*color).clamp().into_format().into_components();
            queue!(stdout, SetForegroundColor(if color.l > 50.0 { Color::Black } else { Color::White }))?;
            queue!(stdout, SetBackgroundColor(Color::Rgb {
                r: col.0, g: col.1, b: col.2
            }), Print(tx))?;
            cur_width += 16;
            if cur_width + 16 > width {
                queue!(stdout, Print("\n"))?;
                cur_width = 0;
            }
        }
        queue!(stdout, ResetColor, Print("\n"))?;
    }

    stdout.flush()?;
    Ok(())
}

pub fn eval(palette: &Palette, expr: String, colored: bool, output_format: String) -> Result<()> {
    let color = crate::palette::color_parser::color(&expr)?.resolve(palette)?;
    let col: (u8,u8,u8,u8) = palette::Srgba::from_color(color).clamp().into_format().into_components();
    use crate::expander::ColorOutputRep;
    let output_type = match output_format.chars().next() {
        Some('#') => ColorOutputRep::Hash(false),
        Some('~') => ColorOutputRep::LinHash(false),
        Some('$') => ColorOutputRep::CssRgb,
        Some('!') => ColorOutputRep::CssLch,
        _ => bail!("invalid output format {}", output_format)
    };
    let with_alpha = output_format.chars().nth(1).map_or(false, |c| c == 'a');
    let mut stdout = std::io::stdout();
    if colored {
        queue!(stdout, SetForegroundColor(if color.l > 50.0 { Color::Black } else { Color::White }))?;
        queue!(stdout, SetBackgroundColor(Color::Rgb {
            r: col.0, g: col.1, b: col.2
        }))?;
    }
    queue!(stdout, Print(crate::expander::fmt_color(color, output_type, with_alpha)), ResetColor)?;
    stdout.flush()?;
    Ok(())
}
