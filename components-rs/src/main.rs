// Components: custom components and their props, as a small team settings page.
//
// - components/badge.rs:   required and defaulted props, a reactive `Prop<T>`, and an enum whose
//                          variants are leading-dot names (`#[derive(DotValues)]`)
// - components/avatar.rs:  a numeric default and an `Option` prop
// - components/action.rs:  a prop type of your own, with `From` so closures can be passed
// - components/member.rs:  a component made of other components, with a callback prop
// - components/setting.rs: a `Binding` prop that changes its parent's state
// - components/card.rs:    a struct component that takes children, with builder methods, and
//                          a modifier only cards have (`.highlighted(color: .accent)`)
// - components/outline.rs: a modifier every view has (`.outlined(radius: 10)`), with named
//                          arguments
// - root.rs:               the page, putting them together

mod components;
mod root;

use std::process::ExitCode;

use elegance::{App, Error, WindowDesc};

fn main() -> Result<ExitCode, Error> {
    App::new()
        .window(WindowDesc::new("Components").size(560, 720).min_size(420.0, 480.0), root::Root)
        .run()
}
