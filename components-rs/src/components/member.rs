// One person on the team: a component built from other components (Avatar, Badge), with a
// callback prop (`on_remove`) the parent uses to change its list.

use elegance::prelude::*;

use super::{Action, Avatar, Badge, Density, Tone};

#[derive(Clone, Copy, Debug, PartialEq, DotValues)]
pub enum Role {
    Owner,
    Admin,
    Member,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: u32,
    pub name: String,
    pub role: Role,
    pub color: Color,
}

impl Member {
    fn initials(&self) -> String {
        self.name.split_whitespace().filter_map(|word| word.chars().next()).take(2).collect()
    }
}

#[component]
pub fn MemberRow(
    member: Member,
    #[prop(default = .comfortable)] density: Density,
    on_remove: Action,
) -> impl IntoView {
    let (role, tone) = match member.role {
        Role::Owner => ("Owner", Tone::Accent),
        Role::Admin => ("Admin", Tone::Warning),
        Role::Member => ("Member", Tone::Neutral),
    };
    let avatar_size: f32 = match density {
        Density::Compact => 28.0,
        Density::Comfortable => 40.0,
    };
    let removable = member.role != Role::Owner;

    elg! {
        HStack(spacing: 12) {
            Avatar(initials: member.initials(), size: avatar_size, color: member.color)
            Text(member.name.clone())
            Spacer()
            Badge(text: role, tone: tone)
            Button(Image("trash"), move || on_remove.run())
                .button_style(PlainButtonStyle)
                .foreground_style(.danger)
                .disabled(!removable)
                .opacity(if removable { 1.0 } else { 0.0 })
                .accessibility_label("Remove")
        }
    }
}
