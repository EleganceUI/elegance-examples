// Todos: a list in state. TextField and Binding, ForEach keyed by id, If, Computed, and
// animated insertions and removals.

#include <elg/elegance.h>

#include <algorithm>
#include <format>
#include <string>
#include <vector>

using namespace elg;

namespace {

struct Todo {
    int id = 0;
    std::string title;
    bool done = false;

    friend bool operator==(const Todo&, const Todo&) = default;
};

using Todos = State<std::vector<Todo>>;

View TodoRow(int id, std::string title, Todos todos) {
    // A Binding reads and writes one field of one item in the list.
    const Binding<bool> done(
        [=] {
            const auto& list = todos();
            const auto it = std::ranges::find(list, id, &Todo::id);
            return it != list.end() && it->done;
        },
        [=](bool value) {
            todos.update([&](std::vector<Todo>& list) {
                if (const auto it = std::ranges::find(list, id, &Todo::id); it != list.end()) {
                    it->done = value;
                }
            });
        });

    const auto remove = [=] {
        with_animation(Animation::smooth(), [&] {
            todos.update([&](std::vector<Todo>& list) { std::erase_if(list, [&](const Todo& t) { return t.id == id; }); });
        });
    };

    return HStack(Checkbox(std::move(title), done),
                  Spacer(),
                  Button(Image("trash"), remove)
                      .button_style(PlainButtonStyle{})
                      .foreground_style(ThemeColor::danger)
                      .accessibility_label("Delete"));
}

View TodoApp() {
    const Todos todos(std::vector<Todo>{{1, "Read the docs"}, {2, "Try the examples"}, {3, "Build something"}});
    const State<std::string> draft;
    const State<int> next_id(4);
    const auto remaining = computed([=] { return std::ranges::count(todos(), false, &Todo::done); });

    const auto add = [=] {
        if (draft().empty()) {
            return;
        }
        with_animation(Animation::smooth(), [&] {
            todos.update([&](std::vector<Todo>& list) { list.push_back({next_id(), draft()}); });
        });
        next_id += 1;
        draft = std::string();
    };

    return VStack({.alignment = HorizontalAlignment::leading, .spacing = 12},
                  Text("Todos").font(Font::large_title),
                  HStack(TextField("What needs doing?", draft, {.on_submit = add}),
                         Button("Add", add).disabled([=] { return draft().empty(); })),
                  If([=] { return todos().empty(); },
                     Text("Nothing left to do.").foreground_style(ThemeColor::secondary),
                     ScrollView(LazyVStack({.spacing = 8, .separators = StackSeparators{}},
                                           ForEach(todos, &Todo::id, [=](const Todo& todo) {
                                               return TodoRow(todo.id, todo.title, todos);
                                           })))),
                  Text([=] { return std::format("{} left", remaining()); })
                      .font(Font::caption)
                      .foreground_style(ThemeColor::secondary))
        .padding(24)
        .frame({.max_width = infinity, .max_height = infinity, .alignment = Alignment::top_leading});
}

} // namespace

int main() {
    App app;
    app.window({.title = "Todos", .width = 480, .height = 560, .min_size = Size{360, 320}}, TodoApp);
    return app.run();
}
