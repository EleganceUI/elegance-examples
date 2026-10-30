// A circle with someone's initials. `size` defaults to a number; `color` is an `Option`, which
// defaults to `None` by itself, and callers pass a plain `Color` (it converts into `Some`).

use elegance::prelude::*;

#[component]
pub fn Avatar(initials: String, #[prop(default = 36.0)] size: f32, color: Option<Color>) -> impl IntoView {
    let fill = color.unwrap_or(Color::hex(0x8E8E93));
    elg! {
        ZStack {
            Circle().fill(fill)
            Text(initials.clone())
                .font(Font::system(size * 0.4, .semibold))
                .foreground_style(.white)
        }
        .frame(size, size)
    }
}
