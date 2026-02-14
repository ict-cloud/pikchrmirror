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
