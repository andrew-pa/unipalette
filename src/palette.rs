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
    WithLightness(Box<ColorSpec<'s>>, f32),
    WithChroma(Box<ColorSpec<'s>>, f32),
    WithAlpha(Box<ColorSpec<'s>>, f32),
    Mix(Box<ColorSpec<'s>>, Box<ColorSpec<'s>>, f32),
    Complement(Box<ColorSpec<'s>>),
    FnCall(&'s str, Vec<ColorSpec<'s>>)
}

#[derive(Debug)]
pub struct ColorFn<'s> {
    args: Vec<&'s str>,
    body: ColorSpec<'s>
}

pub enum PaletteItem<'s> {
    Color(&'s str, ColorSpec<'s>),
    Func(&'s str, ColorFn<'s>)
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
            fnname:name() whitespace()? "(" whitespace()? args:(color() ** ("," whitespace()?)) ")" { ColorSpec::FnCall(fnname, args) }
            --
            "~" a:(@) { ColorSpec::Complement(Box::new(a)) }
            --
            a:(@) whitespace()? "*" c:number() "*" whitespace()? b:@ { ColorSpec::Mix(Box::new(a), Box::new(b), c) }
            --
            c:@ whitespace() "ch" p:number() { ColorSpec::WithChroma(Box::new(c), p) }
            c:@ whitespace() "st" p:number() { ColorSpec::Saturate(Box::new(c), p) }
            c:@ whitespace() "li" x:number() { ColorSpec::Shade(Box::new(c), x) }
            c:@ whitespace() "li=" x:number() { ColorSpec::WithLightness(Box::new(c), x) }
            x:@ whitespace()? "a" a:number() { ColorSpec::WithAlpha(Box::new(x), a) }
            --
            lch:lch_literal() { ColorSpec::Lch(lch) }
            id:name() { ColorSpec::Id(id) }
            "$" n:name() { ColorSpec::Named(n) }
            "(" c:color() ")" { c }
        }

        rule color_item() -> PaletteItem<'input> = n:name() whitespace()? "=" whitespace()? c:color() { PaletteItem::Color(n, c) }

        rule func_item() -> PaletteItem<'input> =
            "fn" whitespace() name:name() whitespace()?
            "(" whitespace()? args:(name() ** ("," whitespace()?)) ")" whitespace()?
            "=" whitespace()? body:color()
            { PaletteItem::Func(name, ColorFn { args, body }) };

        pub rule palette_def() -> PaletteItem<'input> = color_item() / func_item()
    }
}

pub struct Palette<'s> {
    pub colors: HashMap<&'s str, Lcha>,
    pub functions: HashMap<&'s str, ColorFn<'s>>
}

impl<'s> ColorSpec<'s> {
    pub fn resolve(&self, palette: &Palette) -> Result<Lcha> {
        self.resolve_b(palette, None)
    }

    fn resolve_b(&self, palette: &Palette, local_bindings: Option<&HashMap<&'s str, Lcha>>) -> Result<Lcha> {
        use palette::{FromColor, Shade, Saturate, Mix};
        match self {
            ColorSpec::Id(i) => {
                local_bindings.and_then(|lb| lb.get(*i).cloned())
                    .or_else(|| palette.colors.get(*i).cloned()).ok_or(anyhow!("unknown id {}", i))
            },
            ColorSpec::Named(n) => palette::named::from_str(*n)
                .map(|c| {
                    let col = palette::Lch::from_rgb(c.into_format().into_linear());
                    Lcha::new(col.l, col.chroma, col.hue, 1.0)
                } )
                .ok_or(anyhow!("unknown color {}", n)),
            ColorSpec::Lch(c) => Ok(*c),
            ColorSpec::Shade(c, p) => c.resolve_b(palette, local_bindings).map(|c| c.lighten(*p / 100.0)),
            ColorSpec::Saturate(c, p) => c.resolve_b(palette, local_bindings).map(|c| c.saturate(*p / 100.0)),
            ColorSpec::WithChroma(c, p) => c.resolve_b(palette, local_bindings).map(|mut c| {c.chroma = *p; return c;}),
            ColorSpec::WithAlpha(c, p) => c.resolve_b(palette, local_bindings).map(|mut c| {c.alpha = *p/100.0; return c;}),
            ColorSpec::WithLightness(c, p) => c.resolve_b(palette, local_bindings).map(|mut c| {c.l = *p; return c;}),
            ColorSpec::Mix(a, b, f) => a.resolve_b(palette, local_bindings).and_then(|a| b.resolve_b(palette, local_bindings).map(|b| (a,b))).map(|(a,b)| a.mix(&b, *f)),
            ColorSpec::Complement(c) => c.resolve_b(palette, local_bindings).map(|mut c| { c.hue += 180.0; return c;}),
            ColorSpec::FnCall(name, args) => {
                palette.functions.get(name).ok_or_else(|| anyhow!("unknown function {}", name))
                    .and_then(|f| {
                        let mut arg_bindings = HashMap::new();
                        for (name, color) in args.iter().zip(f.args.iter())
                            .map(|(expr, name)| (name, expr.resolve_b(palette, local_bindings)))
                        {
                            arg_bindings.insert(*name, color?);
                        }
                        f.body.resolve_b(palette, Some(&arg_bindings))
                    })
            }
        }
    }
}

pub fn read_palette<'s>(src: &'s str) -> Result<Palette<'s>> {
    let mut p = Palette { colors: HashMap::new(), functions: HashMap::new() };
    for (ix, ln) in src.lines().enumerate() {
        if ln.starts_with('#') || ln.len() == 0 { continue; }
        match color_parser::palette_def(ln)? {
            PaletteItem::Color(name, spec) => {
                p.colors.insert(name, spec.resolve(&p)?);
            },
            PaletteItem::Func(name, f) => {
                p.functions.insert(name, f);
            }
        }
    }
    Ok(p)
}
