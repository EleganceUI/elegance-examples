// A titled panel around whatever views are put in its block:
//
//     Card(title: "Members", density: .compact) { ... }
//         .highlighted(color: .accent, width: 2)
//
// A component written as a struct instead of with #[component], for when it takes children:
// - `#[elg_builder(constructor)]` lets elg! make it with `Card(...)` (it calls `new`), and makes
//   `.compact` work for `density` even though it takes exactly a `Density`, not `impl Into`.
// - The builder methods are its named arguments.
// - `ElgChildren` receives the views in its block.
// - `Component` builds it.
//
// `.highlighted(...)` is a modifier only cards have: a method on `Card`, not on every view (see
// outline.rs for one every view has). It recolors the title, which only the card can reach.
// In elg!, `.name(key: v)` calls `.name_with(|o| o.key(v))`, so the named arguments are the
// builder methods of `Highlight`, the options `highlighted_with` takes. It works where the
// expression is still a `Card`: put it before modifiers that make the card a plain `View`.

use elegance::dsl::ElgChildren;
use elegance::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, DotValues)]
pub enum Density {
    Compact,
    Comfortable,
}

/// How a highlighted card looks: `.highlighted(color: .accent, width: 2)`.
#[derive(Clone, Copy)]
pub struct Highlight {
    color: ThemeColor,
    width: f32,
}

impl Default for Highlight {
    fn default() -> Self {
        Self {
            color: ThemeColor::Accent,
            width: 1.0,
        }
    }
}

#[elg_builder]
impl Highlight {
    pub fn color(mut self, color: ThemeColor) -> Self {
        self.color = color;
        self
    }
    pub fn width(mut self, width: impl Number) -> Self {
        self.width = width.to_f32();
        self
    }
}

pub struct Card {
    title: String,
    density: Density,
    highlight: Option<Highlight>,
    children: Vec<View>,
}

#[elg_builder(constructor)]
impl Card {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            density: Density::Comfortable,
            highlight: None,
            children: Vec::new(),
        }
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
    /// `.highlighted(color: .accent, width: 2)` in elg!.
    pub fn highlighted_with(mut self, options: impl FnOnce(Highlight) -> Highlight) -> Self {
        self.highlight = Some(options(Highlight::default()));
        self
    }
    /// `.highlighted()`: the default highlight.
    pub fn highlighted(self) -> Self {
        self.highlighted_with(|options| options)
    }
}

impl ElgChildren for Card {
    type Out = Card;
    fn elg_children(mut self, children: Vec<View>) -> Card {
        self.children = children;
        self
    }
}

impl Component for Card {
    fn body(&self) -> impl IntoView {
        let spacing = match self.density {
            Density::Compact => 8,
            Density::Comfortable => 14,
        };
        let rows = VStack::new()
            .alignment(HorizontalAlignment::Leading)
            .spacing(spacing)
            .children(self.children.clone());
        let title_color = self.highlight.map_or(ThemeColor::Primary, |h| h.color);

        let card = elg! {
            VStack(alignment: .leading, spacing: 12) {
                Text(self.title.clone()).font(.headline).foreground_style(title_color)
                rows
            }
            .padding(16)
            .frame(max_width: .infinity, alignment: .leading)
            .background_in(.surface, RoundedRectangle(12))
        };
        match self.highlight {
            Some(h) => card.overlay(RoundedRectangle(12).stroke_border(h.color, h.width)),
            None => card.into_view(),
        }
    }
}
