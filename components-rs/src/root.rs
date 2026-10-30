// The page: owns the state, and passes it down to the components as props.

use elegance::prelude::*;

use crate::components::{Badge, Card, Density, Member, MemberRow, Outlined, Role, SettingRow};

fn starting_team() -> Vec<Member> {
    vec![
        Member {
            id: 1,
            name: "Ada Lovelace".into(),
            role: Role::Owner,
            color: Color::hex(0x5E5CE6),
        },
        Member {
            id: 2,
            name: "Grace Hopper".into(),
            role: Role::Admin,
            color: Color::hex(0xFF9F0A),
        },
        Member {
            id: 3,
            name: "Alan Turing".into(),
            role: Role::Member,
            color: Color::hex(0x30D158),
        },
        Member {
            id: 4,
            name: "Katherine Johnson".into(),
            role: Role::Member,
            color: Color::hex(0xFF375F),
        },
    ]
}

const NEW_NAMES: [&str; 4] = [
    "Edsger Dijkstra",
    "Barbara Liskov",
    "Donald Knuth",
    "Margaret Hamilton",
];

#[component]
pub fn Root() -> impl IntoView {
    let members = State::new(starting_team());
    let density = State::new(Density::Comfortable);
    let notifications = State::new(true);
    let weekly_digest = State::new(false);
    let mut next_id = State::new(5_u32);

    let count = members.map(Vec::len);

    let invite = move || {
        let id = next_id.get();
        let name = NEW_NAMES[id as usize % NEW_NAMES.len()];
        with_animation(Animation::spring(0.3, 0.5), || {
            members.write().push(Member {
                id,
                name: name.into(),
                role: Role::Member,
                color: Color::hex(0x64D2FF),
            });
        });
        next_id += 1;
    };

    elg! {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                HStack(spacing: 10) {
                    Text("Team").font(.large_title)
                    // A Prop<String> prop: this text updates as members come and go.
                    Badge(text: "{count} members", tone: .accent)
                    Spacer()
                    Button("Invite", invite)
                }

                Picker(density, [(Density::Compact, "Compact"), (Density::Comfortable, "Comfortable")])
                    .outlined()

                // `density` is a plain value prop, so the card is rebuilt when the State changes.
                match density.get() {
                    current => Card(title: "Members", density: current) {
                        ForEach(members, id: |m| m.id) { member in
                            MemberRow(
                                member: member.clone(),
                                density: current,
                                on_remove: {
                                    let id = member.id;
                                    move || with_animation(Animation::spring(0.3, 0.5), || {
                                        members.write().retain(|m| m.id != id)
                                    })
                                },
                            )
                        }
                    }
                    // A modifier only cards have. No arguments: the options' defaults.
                    .highlighted()
                    .clipped()
                }

                // The card's own density, set with a leading dot.
                Card(title: "Notifications", density: .compact) {
                    SettingRow(title: "Email notifications", note: "Mentions and replies.".to_string(), is_on: notifications)
                    SettingRow(title: "Weekly digest", is_on: weekly_digest)
                }
                .highlighted(color: .accent, width: 2)

                // The rows above changed this State: Binding props are two-way.
                HStack(spacing: 8) {
                    // `text` is reactive (a Prop), `tone` is a plain value set once.
                    Badge(text: move || if notifications.get() { "Notifications on" } else { "Notifications off" },
                          tone: .success)
                    Badge(text: "Digest", tone: .warning).opacity(move || if weekly_digest.get() { 1.0 } else { 0.3 })
                }
                .padding(10)
                // A modifier every view has, with named arguments.
                .outlined(color: .outline, radius: 10, width: 1)
            }
            .padding(24)
            .frame(max_width: .infinity, alignment: .leading)
        }
    }
}
