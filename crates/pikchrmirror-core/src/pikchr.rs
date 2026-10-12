use pikchr::{Pikchr, PikchrFlags};

pub fn pik_svgstring(i_raw: &str, i_svg_old: &str) -> (String, String) {
    let mut pik_err = String::from("");
    let svg_str = match Pikchr::render(i_raw, None, PikchrFlags::default()) {
        Ok(p) => p.rendered().to_owned(),
        Err(e) => {
            pik_err = e.to_owned();
            i_svg_old.to_owned()
        }
    };

    (svg_str, pik_err)
}

/// A successfully rendered diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub svg: String,
    pub width: i64,
    pub height: i64,
}

/// A pikchr compile error with its position, when pikchr reported one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
    /// 1-based line of the offending token.
    pub line: Option<usize>,
    /// 1-based column of the offending token.
    pub col: Option<usize>,
    /// The raw pikchr error text (source excerpt, caret marker, macro trace).
    pub context: String,
}

/// Width of the `/* %4d */  ` prefix pikchr puts in front of each echoed source line.
const LINE_PREFIX_WIDTH: usize = 12;

fn parse_line_number(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("/* ")?;
    let (num, _) = rest.split_once(" */")?;
    num.trim().parse().ok()
}

/// Extracts message, line and column from pikchr's plain-text error output.
///
/// Positioned errors look like `/* NNNN */  <source line>`, a caret line, then `ERROR: <msg>`.
/// Errors without a token (e.g. parser stack overflow) carry no position.
pub fn parse_error(raw: &str) -> CompileError {
    let lines: Vec<&str> = raw.lines().collect();
    let context = raw.trim().to_string();

    let Some(err_idx) = lines.iter().position(|l| l.starts_with("ERROR: ")) else {
        return CompileError {
            message: context.clone(),
            line: None,
            col: None,
            context,
        };
    };
    let message = lines[err_idx]["ERROR: ".len()..].trim().to_string();

    let caret_idx = err_idx.checked_sub(1).filter(|&i| {
        lines[i].contains('^') && lines[i].trim_matches(|c| c == ' ' || c == '^').is_empty()
    });
    let (line, col) = match caret_idx {
        Some(ci) => {
            let col = lines[ci]
                .find('^')
                .map(|spaces| spaces.saturating_sub(LINE_PREFIX_WIDTH) + 1);
            let line = lines[..ci].iter().rev().find_map(|l| parse_line_number(l));
            (line, col)
        }
        None => (None, None),
    };

    CompileError {
        message,
        line,
        col,
        context,
    }
}

/// Compiles pikchr source to SVG, reporting failures as a structured [`CompileError`].
pub fn render_svg(source: &str) -> Result<Rendered, CompileError> {
    match Pikchr::render(source, None, PikchrFlags::default()) {
        Ok(p) => Ok(Rendered {
            svg: p.rendered().to_owned(),
            width: p.width() as i64,
            height: p.height() as i64,
        }),
        Err(e) => Err(parse_error(&e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pik_svgstring_success_returns_svg_with_no_error() {
        let (svg, error) = pik_svgstring("box", "");
        assert!(svg.contains("<svg"));
        assert!(error.is_empty());
    }

    #[test]
    fn test_pik_svgstring_error_keeps_previous_svg_and_reports_error() {
        let old_svg = "<svg>previous</svg>";
        let (svg, error) = pik_svgstring("box \"unterminated", old_svg);
        assert_eq!(svg, old_svg);
        assert!(!error.is_empty());
    }

    #[test]
    fn test_render_svg_success() {
        let r = render_svg("box \"hi\"").unwrap();
        assert!(r.svg.contains("<svg"));
        assert!(r.width > 0 && r.height > 0);
    }

    #[test]
    fn test_render_svg_reports_line_and_column() {
        let err = render_svg("box\narrow\nbogus_token_here\n").unwrap_err();
        assert!(!err.message.is_empty());
        assert_eq!(err.line, Some(3), "context: {}", err.context);
        assert!(err.col.is_some(), "context: {}", err.context);
    }

    #[test]
    fn test_render_svg_column_matches_real_pikchr_output() {
        // `"B"` is the offending token and starts at 1-based column 6 of line 3.
        let err = render_svg("box \"A\"\narrow\nboks \"B\"\n").unwrap_err();
        assert_eq!(
            (err.line, err.col),
            (Some(3), Some(6)),
            "context: {}",
            err.context
        );

        // Error on the very first token of the first line.
        let err = render_svg("@@@").unwrap_err();
        assert_eq!(err.line, Some(1), "context: {}", err.context);
        assert_eq!(err.col, Some(1), "context: {}", err.context);
    }

    #[test]
    fn test_render_svg_unterminated_string_is_an_error() {
        let err = render_svg("box \"unterminated").unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn test_render_svg_empty_source_is_ok() {
        let r = render_svg("").unwrap();
        assert!(r.svg.starts_with("<!-- empty pikchr diagram -->"));
    }

    #[test]
    fn test_parse_error_positioned() {
        let raw = "/*    1 */  box\n/*    2 */  oops\n               ^^^^\nERROR: syntax error\n";
        let e = parse_error(raw);
        assert_eq!(e.message, "syntax error");
        assert_eq!(e.line, Some(2));
        assert_eq!(e.col, Some(4));
    }

    #[test]
    fn test_parse_error_without_position() {
        let e = parse_error("\nparser stack overflow");
        assert_eq!(e.message, "parser stack overflow");
        assert_eq!((e.line, e.col), (None, None));
    }
}
