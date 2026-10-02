# Pikchr syntax cheat sheet

Pikchr is a PIC-like diagram language. A diagram is a list of statements separated by newlines or `;`.
Objects are placed one after another in the current direction (default `right`). `#` starts a comment.
Text is always in double quotes. Call `render_pikchr` to check your source; fix errors using the reported line/col.

## Objects

`box`, `circle`, `ellipse`, `oval`, `cylinder`, `file`, `diamond`, `line`, `arrow`, `spline`, `arc`, `dot`, `text`, `move` (invisible gap).

```pikchr
box "Start" rad 10px fit
arrow
box "Process" fit
arrow
box "End" rad 10px fit
```

## Direction

`right` (default), `left`, `up`, `down` set the flow of following objects. They can also be given per object.

```pikchr
down
box "Top" fit
arrow 60%
box "Middle" fit
arrow 60%
box "Bottom" fit
```

## Common attributes

- Size: `width 2cm`, `height 1cm`, `radius 5mm` (`rad` for corner rounding), `fit` (size to text), percentages like `200%` scale the default (`arrow right 200%`).
- Text: `"label"` (up to a few strings per object, stacked), then `above`, `below`, `ljust`, `rjust`, `bold`, `italic`, `big`, `small`.
- Style: `color red`, `fill lightblue`, `thin`, `thick`, `thickness 3px`, `dashed`, `dotted`, `invis`.
- Arrowheads on `line`: `->`, `<-`, `<->`.
- Units: `in`, `cm`, `mm`, `px`, `pt`, `pc`. A bare number means inches.

```pikchr
box "Client" fit fill lightblue
line <-> right 150% "request" above "response" below
box "Server" fit fill lightgreen
```

## Labels and places

Name an object with a Capitalized label and refer to its corners or edges:
`.n .s .e .w .ne .nw .se .sw .c .start .end`. Also `1st box`, `last circle`, `previous`.

```pikchr
A: box "A" fit
B: box "B" fit at 1.5in right of A
arrow from A.e to B.w
C: circle "C" fit at 1in below A
line from A.s to C.n
```

## Lines and paths

`line from X to Y then to Z`, `arrow to P`, `go right 1in then down 0.5in`. `chop` trims a line to the shapes at its ends.

```pikchr
A: box "A" fit
B: box "B" fit at 2in right of A
C: box "C" fit at 1in below B
arrow from A.e to B.w
arrow from B.s to C.n
arrow from C.w to A.s chop
```

## Grouping and variables

`[ ... ]` groups objects into one box-like unit. `$name = value` defines a variable. `define name { ... }` makes a macro.

```pikchr
$w = 1in
Group: [
  box "One" width $w
  arrow
  box "Two" width $w
]
box "Outside" fit with .n at 0.4in below Group.s
```

## Tips

- Prefer `fit` on boxes with text so nothing overflows.
- Keep one flow direction per diagram and use labels for cross-links.
- A syntax error is reported as `ERROR: <message>` at a source line and column; fix that token, not the whole diagram.
- Full reference: https://pikchr.org/home/doc/trunk/doc/userman.md
