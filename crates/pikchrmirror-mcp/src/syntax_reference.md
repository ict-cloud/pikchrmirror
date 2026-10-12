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

## Formal grammar

Complete grammar from https://pikchr.org/home/doc/trunk/doc/grammar.md. Italic names are rules, bold or quoted tokens are literal,
`?` optional, `*` zero or more, `|` alternatives. UPPERCASE names are lexical tokens (LABEL starts with a capital letter, VARIABLE is a lowercase name or `$name`, ORDINAL is `1st`, `2nd`, ...).

```text
statement-list:
    statement?
    statement-list NEWLINE statement?
    statement-list ; statement?

statement:
    object-definition
    LABEL : object-definition
    LABEL : place
    direction
    VARIABLE assignment-op expr
    define VARIABLE CODEBLOCK
    print print-argument (, print-argument)*
    assert ( expr == expr )
    assert ( position == position )

direction:        right | down | left | up
assignment-op:    = | += | -= | *= | /=
print-argument:   expr | STRING

object-definition:
    object-class attribute*
    STRING text-attribute* attribute*
    [ statement-list ] attribute*

object-class:
    arc | arrow | box | circle | cylinder | diamond | dot | ellipse | file
    | line | move | oval | spline | text

attribute:
    path-attribute
    location-attribute
    STRING text-attribute*
    same | same as object
    numeric-property new-property-value
    dashed expr? | dotted expr?
    color color-expr | fill color-expr
    behind object
    cw | ccw
    <- | -> | <->
    invis | invisible
    thick | thin | solid
    chop | fit

color-expr:          expr
new-property-value:  expr | expr %
numeric-property:    diameter | ht | height | rad | radius | thickness | width | wid

text-attribute:
    above | aligned | below | big | bold | mono | monospace | center
    | italic | ljust | rjust | small

path-attribute:
    from position
    then? to position
    then? go? direction line-length?
    then? go? direction until? even with position
    (then|go) line-length? heading compass-angle
    (then|go) line-length? compass-direction
    close

line-length:       expr | expr %
compass-angle:     expr
compass-direction: n | north | ne | e | east | se | s | south | sw | w | west | nw

location-attribute:
    at position
    with edgename at position
    with dot-edgename at position

position:
    expr , expr
    place
    place + expr , expr
    place - expr , expr
    place + ( expr , expr )
    place - ( expr , expr )
    ( position , position )
    ( position )
    fraction of the way between position and position
    fraction way between position and position
    fraction between position and position
    fraction < position , position >
    distance which-way-from position

fraction:  expr
distance:  expr

which-way-from:
    above | below | right of | left of
    n of | north of | ne of | e of | east of | se of | s of | south of
        | sw of | w of | west of | nw of
    heading compass-angle from

place:
    object
    object dot-edgename
    edgename of object
    ORDINAL vertex of object

object:
    LABEL
    object . LABEL
    nth-object of|in object

nth-object:
    ORDINAL object-class
    ORDINAL last object-class
    ORDINAL previous object-class
    last object-class | previous object-class
    last | previous
    ORDINAL [] | ORDINAL last [] | ORDINAL previous []
    last [] | previous []

dot-edgename:
    .n | .north | .t | .top | .ne | .e | .east | .right | .se | .s | .south
    | .bot | .bottom | .sw | .w | .west | .left | .nw | .c | .center
    | .start | .end

edgename:
    n | north | ne | e | east | se | s | south | sw | w | west | nw
    | t | top | bot | bottom | left | right | c | center | start | end

expr:
    NUMBER | VARIABLE | COLORNAME
    place .x | place .y
    object dot-property
    ( expr )
    expr + expr | expr - expr | expr * expr | expr / expr
    - expr | + expr
    abs ( expr )
    cos ( expr ) | sin ( expr )
    dist ( position , position )
    int ( expr )
    max ( expr , expr ) | min ( expr , expr )
    sqrt ( expr )

dot-property:
    .color | .dashed | .diameter | .dotted | .fill | .ht | .height | .rad
    | .radius | .thickness | .wid | .width
```
