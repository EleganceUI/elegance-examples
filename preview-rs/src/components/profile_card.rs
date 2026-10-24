// A person's name and role beside an avatar with their initials, and its previews.

use elegance::prelude::*;

fn initials(name: &str) -> String {
    name.split_whitespace().filter_map(|word| word.chars().next()).collect()
}

#[component]
pub fn ProfileCard(name: String, role: String) -> impl IntoView {
    let initials = initials(&name);

    elg! {
        HStack(spacing: 14) {
            Circle()
                .fill(.accent)
                .frame(48, 48)
                .overlay(Text(initials.clone()).font(.headline).foreground_style(.white))
            VStack(alignment: .leading, spacing: 2) {
                Text(name.clone()).font(.headline)
                Text(role.clone()).foreground_style(.secondary)
            }
            Spacer()
        }
        .padding(16)
        .frame(width: 320)
        .background_in(.surface, RoundedRectangle(14))
    }
}

// Previews: the card on its own, with sample data. `cargo elegance preview` lists them.

#[preview("Profile card")]
fn profile_card() -> impl IntoView {
    elg! { ProfileCard(name: "Ada Lovelace", role: "Analyst").padding(24) }
}

#[preview("Profile card, dark")]
fn profile_card_dark() -> impl IntoView {
    elg! {
        ProfileCard(name: "Grace Hopper", role: "Rear admiral")
            .padding(24)
            .frame(max_width: .infinity, max_height: .infinity)
            .background(.background)
            .color_scheme(.dark)
    }
}
