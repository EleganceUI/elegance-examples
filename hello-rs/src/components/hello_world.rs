// A component with a prop (`msg`) and some state of its own (`count`).

use elegence::prelude::*;

#[component]
pub fn HelloWorld(msg: String) -> impl IntoView {
    let mut count = State::new(0);

    elg! {
        VStack(spacing: 16) {
            Text(msg.clone()).font(.large_title)
            Button("count is {count}", move || count += 1)
            Text("Edit src/components/hello_world.rs and run again to see changes.")
                .foreground_style(.secondary)
        }
    }
}
