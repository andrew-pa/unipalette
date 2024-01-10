# Unipalette

## Usage

`unipalette <color palette path> [subcommand]`

Subcommands:

- `preview`

    Show colors from the palette in the terminal. If `--shades` is specified, 2 shades ligher and darker will also be displayed.

- `eval <expression>`

    Evaluates a color expression from the command line and prints the resulting color. If `-c` or `--colored` is specified, then the color will be used to color the text as well. Output formats can be specified with `-o` using the same characters as in the Syntax section about expanding color references, the default is sRGB hex.

- `expand <path>`

    Process the file at <path> or all files ending in `.uncol` under <path>, expanding color references as explained in the Syntax section, using the palette specified. When processing directories, the resulting expanding file will be written in the same directory as the source file, but without the `.uncol` extention

## Syntax

### Color Expressions
Literal colors can either be referred to as [LCH](https://en.wikipedia.org/wiki/CIELAB_color_space#Cylindrical_model) triplets or by using [CSS color names](https://www.w3.org/TR/SVG/types.html#ColorKeywords):
```
# an lch color
purple=l40c120h300

# a css color
$purple
```

Operators can affect colors, changing their lightness or saturation.
```
# purple from above, but 50% more saturated
purple st+50
# purple from above, but 50% less saturated
purple st-50

# purple from above, but 50% darker
purple li-50
# purple from above, but 50% lighter
purple li+50

# set the chroma value directly, making a gray
purple ch10

# set the lightness value directly
purple li=50
```

Colors can have a specified alpha value as well. By default all colors are opaque.
```
#  purple from above with 50% transparency
purple a50
```

Colors can be mixed using the mix operator:
```
# the color halfway linearly between purple and yellow
purple *0.5* $yellow
```
Sometimes the mix operator can have surprising results, because it is linear interpolation.

You can compute the complement of a color:
```
# cyan is the complement of orange
~$orange
```

Once a function has defined, you can call it to compute a new color:
```
# a lovely yellow
my_mix($red, $green)
```

Parenthesis can also be used to group expressions.

### Color functions
Functions that operate on colors can be defined in palette files in the following way:

```
# fn <name>(<args>) = <body>
fn my_mix(a, b) = a *0.5* b
fn const() = $orange
```

The function body is just any regular color expression. The argument identifiers will be bound with the colors passed in when the function is called.
You cannot define a function inside a color reference in a different file, but you can call them.

### Color palettes
A color palette file consists of a list of color or function definitions, one per line. `#` can be used to comment out lines. Once defined, the color can be referred to by name.

```
purple=l40c120h300
green=$green li+20
color3=purple *0.5* green
```

### Color references in other files
Colors in other files can be referenced using color expressions delimited by `~~!` and `!` that are expanded using `unipalette expand`. There are a number of output formatting options.
```
~~!(alpha specifier)(format specifier)(color expression)!
```

Format specifier:

- `#`: RGB hex code in sRGB color space, with prepended `#`
- `~`: RGB hex code in linear sRGB color space with prepended `#`
- `$`: CSS RGB triplet `rgb(R%, G%, B%)`
- `!`: CSS LCH triplet `lch(L% C H)`

Alpha specifier:

- `a`: output alpha. When outputting RGB hex codes put the alpha bytes at the end
- `A`: output alpha. When outputting RGB hex codes put the alpha bytes at the beginning

