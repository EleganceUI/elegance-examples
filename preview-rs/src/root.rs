// The root component: the whole window's content.

use elegance::prelude::*;

use crate::components::ProfileCard;

#[component]
pub fn Root() -> impl IntoView {
    let mut clicks = State::new(0);

    elg! {
        VStack(alignment: .leading, spacing: 14) {
            Text("Team").font(.large_title)
            Text("Run `cargo elegance preview`, edit src/components/profile_card.rs, and save.")
                .foreground_style(.secondary)
            ProfileCard(name: "Ada Lovelace", role: "Analyst")
            ProfileCard(name: "Grace Hopper", role: "Rear admiral")
            Button("Clicked {clicks} times", move || clicks += 1)
        }
        .padding(32)
        .frame(max_width: .infinity, max_height: .infinity, alignment: .top_leading)
    }
}

// The whole app, for `cargo elegance preview --root`.
#[preview(root)]
fn app() -> impl IntoView {
    elg! { Root() }
}
