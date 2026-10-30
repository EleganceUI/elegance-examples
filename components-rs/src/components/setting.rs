// A switch with a title and an optional note. `is_on` is a `Binding<bool>`: the parent passes
// its own `State<bool>`, and flipping the switch here changes the parent's state.

use elegance::prelude::*;

#[component]
pub fn SettingRow(title: String, note: Option<String>, is_on: Binding<bool>) -> impl IntoView {
    elg! {
        VStack(alignment: .leading, spacing: 2) {
            Toggle(title.clone(), is_on)
            match note.clone() {
                Some(note) => Text(note).font(.caption).foreground_style(.secondary),
                None => EmptyView(),
            }
        }
    }
}
