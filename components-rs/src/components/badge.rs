// A small colored label. Shows the three kinds of prop:
//
// - `text` is required (leaving it out is a compile error). It's a `Prop<String>`, so it can be
//   a plain string, a State, a closure, or "{interpolated}" text that updates as state changes.
// - `tone` is optional, defaulting to `.neutral`. `Tone` derives `DotValues`, so callers write
//   `tone: .success` instead of `tone: Tone::Success`.

use elegance::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, DotValues)]
pub enum Tone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

impl Tone {
    fn color(self) -> ThemeColor {
        match self {
            Tone::Neutral => ThemeColor::Secondary,
            Tone::Accent => ThemeColor::Accent,
            Tone::Success => ThemeColor::Success,
            Tone::Warning => ThemeColor::Warning,
            Tone::Danger => ThemeColor::Danger,
        }
    }
}

#[component]
pub fn Badge(text: Prop<String>, #[prop(default = .neutral)] tone: Tone) -> impl IntoView {
    elg! {
        Text(text.clone())
            .font(.caption)
            .foreground_style(.on_accent)
            .padding_edges(.horizontal, 8)
            .padding_edges(.vertical, 2)
            .background_in(tone.color(), Capsule())
    }
}
