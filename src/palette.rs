use std::{collections::HashMap, path::Path};
use anyhow::*;

pub type Lcha = palette::Lcha<palette::white_point::D65>;

#[derive(Debug)]
pub enum ColorSpec<'s> {
    Id(&'s str),
    Named(&'s str),
    Lch(Lcha),
    Shade(Box<ColorSpec<'s>>, f32),
    Saturate(Box<ColorSpec<'s>>, f32),
    WithChroma(Box<ColorSpec<'s>>, f32),
    WithAlpha(Box<ColorSpec<'s>>, f32),
    Mix(Box<ColorSpec<'s>>, Box<ColorSpec<'s>>, f32)
}

peg::parser!{
    pub grammar color_parser() for str {
        rule whitespace() = quiet!{[' ' | '\n' | '\t' ]+}

        rule number() -> f32
            = n:$(['+'|'-']?['0'..='9' | '.']+) {? n.parse().or(Err("could not parse number")) }

        rule name() -> &'input str
            = s:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+) { s }

        rule lch_literal() -> Lcha
            = ['l'|'L'] l:number() ['c'|'C'] c:number() ['h'|'H'] h:number() { Lcha::new(l,c,h,1.0) }

        pub rule color() -> ColorSpec<'input> = precedence! {
            a:(@) whitespace()? "*" c:number() "*" whitespace()? b:@ { ColorSpec::Mix(Box::new(a), Box::new(b), c) }
            --
            c:@ whitespace() "ch" p:number() { ColorSpec::WithChroma(Box::new(c), p) }
            c:@ whitespace() "st" p:number() { ColorSpec::Saturate(Box::new(c), p) }
            c:@ whitespace() "li" x:number() { ColorSpec::Shade(Box::new(c), x) }
            x:@ whitespace()? "a" a:number() { ColorSpec::WithAlpha(Box::new(x), a) }
            --
            lch:lch_literal() { ColorSpec::Lch(lch) }
            id:name() { ColorSpec::Id(id) }
            "$" n:name() { ColorSpec::Named(n) }
            "(" c:color() ")" { c }
        }
    }
}

pub struct Palette {
    pub colors: HashMap<String, Lcha>
}

impl<'s> ColorSpec<'s> {
    pub fn resolve(&self, palette: &Palette) -> Result<Lcha> {
        use palette::{FromColor, Shade, Saturate, Mix};
        match self {
            ColorSpec::Id(i) => palette.colors.get(*i).cloned().ok_or(anyhow!("unknown color {}", i)),
            ColorSpec::Named(n) => palette::named::from_str(*n)
                .map(|c| {
                    let col = palette::Lch::from_rgb(c.into_format().into_linear());
                    Lcha::new(col.l, col.chroma, col.hue, 1.0)
                } )
                .ok_or(anyhow!("unknown color {}", n)),
            ColorSpec::Lch(c) => Ok(*c),
            ColorSpec::Shade(c, p) => c.resolve(palette).map(|c| c.lighten(*p / 100.0)),
            ColorSpec::Saturate(c, p) => c.resolve(palette).map(|c| c.saturate(*p / 100.0)),
            ColorSpec::WithChroma(c, p) => c.resolve(palette).map(|mut c| {c.chroma = *p; return c;}),// c.saturate(*p / 100.0)),
            ColorSpec::WithAlpha(c, p) => c.resolve(palette).map(|mut c| {c.alpha = *p/100.0; return c;}),// c.saturate(*p / 100.0)),
            ColorSpec::Mix(a, b, f) => a.resolve(palette).and_then(|a| b.resolve(palette).map(|b| (a,b))).map(|(a,b)| a.mix(&b, *f)),
        }
    }
}

pub fn load_palette(path: impl AsRef<Path>) -> Result<Palette> {
    let mut p = Palette { colors: HashMap::new() };
    for (ix, ln) in std::fs::read_to_string(path)?.lines().enumerate() {
        if ln.starts_with('#') || ln.len() == 0 { continue; }
        let eq_ix = ln.find('=').ok_or(anyhow!("line {} in palette file does not contain '=': {}", ix, ln))?;
        if eq_ix == 0 { bail!("line {} starts with '='", ix); }
        let spec = color_parser::color(&ln[eq_ix+1..])?;
        // dbg!(&spec);
        p.colors.insert(ln[0..eq_ix].to_owned(),
            spec.resolve(&p)?);
    }
    Ok(p)
}
