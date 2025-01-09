mod img;
mod parser;
mod view;

use view::mainview::app_view;

#[cfg(test)]
mod tests;

fn main() {
    pretty_env_logger::init();
    log::debug!("Hello, PikchrMirror!");
    floem::launch(app_view)
}