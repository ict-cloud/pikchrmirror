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
}
