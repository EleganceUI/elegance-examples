// The root component: the whole window's content.

use elegance::prelude::*;

use crate::components::HelloWorld;

#[component]
pub fn Root() -> impl IntoView {
    elg! {
        VStack(spacing: 24) {
            Image("rocket-launch")
                .font(Font::system(64.0, .regular))
                .foreground_style(.accent)
            HelloWorld(msg: "You did it!")
        }
        .padding(32)
        .frame(max_width: .infinity, max_height: .infinity)
    }
}
