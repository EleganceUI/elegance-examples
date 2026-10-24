// The entry point: opens a window showing the root component.

mod components;
mod root;

use std::process::ExitCode;

use elegence::{App, Error, WindowDesc};

fn main() -> Result<ExitCode, Error> {
    App::new().window(WindowDesc::new("Hello Elegence").size(640, 420), root::Root).run()
}
