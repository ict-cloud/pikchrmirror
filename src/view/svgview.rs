use floem::prelude::*;

pub fn svgpreview_container(
    i_preview_sig: &RwSignal<Vec<u8>>
) -> Scroll {
    let preview_sig = i_preview_sig.clone();
    dyn_container(
        move || preview_sig.get(),
        move |pv| {
            img(move || pv.to_vec())
        }
    )
    .scroll()
    .style(|s| s.flex_col().size_full())
}