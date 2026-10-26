// A modifier every view has: `.outlined(color: .outline, radius: 10, width: 1)`.
//
// A trait implemented for everything that's `IntoView` adds the method to every view, as
// Elegance's own modifiers are. (A modifier for one component is a method on that component
// instead: see `.highlighted` in card.rs.)
//
// In elg!, `.name(a, key: v)` calls `.name_with(a, |o| o.key(v))`. So named arguments come from
// an `outlined_with` method that takes a closure over an options struct, and the options'
// builder methods are the names. `#[elg_builder]` lets `.outline` resolve for `color` (which
// takes exactly a `ThemeColor`) and gives editor completion.

use elegance::prelude::*;

#[derive(Clone, Copy)]
pub struct Outline {
    color: ThemeColor,
    radius: f32,
    width: f32,
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            color: ThemeColor::Outline,
            radius: 8.0,
            width: 1.0,
        }
    }
}

#[elg_builder]
impl Outline {
    pub fn color(mut self, color: ThemeColor) -> Self {
        self.color = color;
        self
    }
    pub fn radius(mut self, radius: impl Number) -> Self {
        self.radius = radius.to_f32();
        self
    }
    pub fn width(mut self, width: impl Number) -> Self {
        self.width = width.to_f32();
        self
    }
}

// The modifier itself: a rounded border over the content.
impl ViewModifier for Outline {
    fn body(&self, content: View) -> View {
        content.overlay(RoundedRectangle(self.radius).stroke_border(self.color, self.width))
    }
}

#[elg_builder]
pub trait Outlined: IntoView + Sized {
    fn outlined_with(self, options: impl FnOnce(Outline) -> Outline) -> View {
        self.into_view().modifier(options(Outline::default()))
    }
    fn outlined(self) -> View {
        self.outlined_with(|options| options)
    }
}

impl<V: IntoView> Outlined for V {}
