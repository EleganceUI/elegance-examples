// Preview: views with #[preview] functions, which the preview host shows on their own and reloads
// as you edit. The app itself is an ordinary binary; previews add nothing to it.
//
//   cargo run                         the app
//   cargo elegance preview            the previews (install: see the README one folder up)
//   cargo elegance preview --root     the whole app (root::app), reloaded as you edit

mod components;
mod root;

use std::process::ExitCode;

use elegance::{App, Error, WindowDesc};

fn main() -> Result<ExitCode, Error> {
    App::new().window(WindowDesc::new("Preview").size(760, 560), root::Root).run()
}
