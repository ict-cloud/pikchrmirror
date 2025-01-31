use crate::img::png;
use pikchr::{Pikchr, PikchrFlags};
use xml::escape::escape_str_pcdata;

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

pub fn pik_preview_width(i_rawstr: &str, i_width: f64) -> Vec<u8> {
    let flags = PikchrFlags::default();
    //flags.generate_html_errors();
    let pikrendr = Pikchr::render(i_rawstr, None, flags);
    let svgstr = match pikrendr {
        Ok(p) => p.rendered().to_owned(),
        // this will not display as proper svg
        Err(e) => {
            if !e.starts_with("<!-- empty pikchr diagram -->") {
                let xml_e = escape_str_pcdata(&e);
                let lstr = xml_e
                    .lines()
                    .collect::<Vec<&str>>()
                    .join(r#"</tspan><tspan dy="1.2em" x="10" dx="1em">"#);
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg"><text><tspan dy="1.2em" x="10" dx="1em">{}</tspan></text></svg>"#,
                    lstr
                )
            } else {
                e.as_str().to_owned()
            }
        }
    };
    println!("svgstring before image processing {}", svgstr);
    let prev_width = if i_width > 0.0 { Some(i_width) } else { None };
    let img = png::svg_to_png(&svgstr, prev_width);
    img
}
