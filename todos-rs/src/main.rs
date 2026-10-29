// Todos: a list in state. TextField and Binding, ForEach keyed by id, `if`, derived state
// (`State::map`) driving a popover, and animated insertions and removals. (The Rust version of
// todos-cpp.)

use std::process::ExitCode;

use elegance::{dot::outline, prelude::*};

#[derive(Clone, PartialEq)]
struct Todo {
    id: i64,
    title: String,
    done: bool,
}

#[component]
fn TodoRow(id: i64, title: String, todos: State<Vec<Todo>>) -> impl IntoView {
    let hovered = State::new(false);
    // A Binding reads and writes one field of one item in the list.
    let done = Binding::new(
        move || todos.read().iter().any(|t| t.id == id && t.done),
        move |value| {
            if let Some(todo) = todos.write().iter_mut().find(|t| t.id == id) {
                todo.done = value;
            }
        },
    );

    let remove = move || {
        with_animation(Animation::smooth(0.4), || {
            todos.write().retain(|t| t.id != id)
        })
    };

    elg! {
        HStack {
            Checkbox(title, done)
            Spacer()
            Button(Image("trash"), remove)
                .button_style(PlainButtonStyle)
                .foreground_style(.danger)
                .accessibility_label("Delete")
        }
        .padding(horizontal: 16, vertical: 10)
        .frame(max_width: .infinity, alignment: .leading)
        .background(Prop::from_env(move |env| {
            ShapeStyle::Color(if hovered.get() {
                env.color(ThemeColor::Outline).opacity(0.35)
            } else {
                Color::CLEAR
            })
        }))
        .content_shape(Rectangle())
        .on_pointer_enter(move || hovered.set(true))
        .on_pointer_leave(move || hovered.set(false))
        .overlay_with(
            elg! {
                Rectangle().fill(.outline)
                    .frame(height: 1)
                    .padding(leading: 16)
                    .opacity(move || if todos.read().last().is_some_and(|todo| todo.id == id) { 0.0 } else { 1.0 })
                    .allows_hit_testing(false)
            },
            |o| o.alignment(.bottom),
        )
    }
}

#[component]
fn TodoApp() -> impl IntoView {
    let todos = State::new(vec![
        Todo {
            id: 1,
            title: "Read the docs".into(),
            done: false,
        },
        Todo {
            id: 2,
            title: "Try the examples".into(),
            done: false,
        },
        Todo {
            id: 3,
            title: "Build something".into(),
            done: false,
        },
    ]);
    let draft = State::new(String::new());
    let mut next_id = State::new(4_i64);
    let remaining = todos.map(|list| list.iter().filter(|t| !t.done).count());

    let add = move || {
        if draft.read().is_empty() {
            return;
        }
        with_animation(Animation::smooth(0.4), || {
            todos.write().push(Todo {
                id: next_id.get(),
                title: draft.get(),
                done: false,
            });
        });
        next_id += 1;
        draft.set(String::new());
    };

    elg! {
        VStack(alignment: .leading, spacing: 12) {
            Text("Todos").font(.large_title)
            HStack {
                TextField("What needs doing?", draft, on_submit: add)
                Button("Add", add).disabled(draft.map(String::is_empty))
                    .popover(todos.map(Vec::is_empty), || elg! {
                        Text("All done! Add another one here.")
                    })
            }
            if todos.read().is_empty() {
                Text("Nothing left to do.").foreground_style(.secondary)
            }
            // A regular stack lets ForEach retain exiting rows until their transitions finish.
            // Keep it mounted even when empty so the final row can animate out.
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(todos, id: |t| t.id) { todo in
                        TodoRow(id: todo.id, title: todo.title.clone(), todos: todos)
                            .transition(Transition::opacity())
                    }
                }
                .background_in(.surface, RoundedRectangle(10))
                .corner_radius(12)
            }
            Text("{remaining} left")
                .font(.caption)
                .foreground_style(.secondary)
        }
        .padding(24)
        .frame(max_width: .infinity, max_height: .infinity, alignment: .top_leading)
    }
}

fn main() -> Result<ExitCode, Error> {
    App::new()
        .window(
            WindowDesc::new("Todos")
                .size(480, 560)
                .min_size(360.0, 320.0),
            TodoApp,
        )
        .run()
}
