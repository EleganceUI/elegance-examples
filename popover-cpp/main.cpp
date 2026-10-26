// Popover: panels shown beside a view while a binding is true. Clicking outside or Esc closes
// them and sets the binding back.

#include <elg/elegance.h>

#include <format>
#include <string>

using namespace elg;

namespace {

View PopoverApp() {
    const State<bool> editing(false);
    const State<bool> showing_info(false);
    const State<std::string> name("Untitled");
    const State<bool> shared(false);

    const View editor = VStack({.alignment = HorizontalAlignment::leading, .spacing = 10},
                               Text("Rename").font(Font::headline),
                               TextField("Name", name),
                               Toggle("Shared with the team", shared.binding()),
                               HStack(Spacer(), Button("Done", [=] { editing = false; })))
                            .frame({.width = 240});

    return VStack({.spacing = 16},
                  Text([=] { return std::format("{}{}", name(), shared() ? " (shared)" : ""); })
                      .font(Font::title),
                  HStack({.spacing = 12},
                         Button("Edit…", [=] { editing = true; }).popover(editing.binding(), [=] { return editor; }),
                         Button(Image("info"), [=] { showing_info = !showing_info(); })
                             .button_style(PlainButtonStyle{})
                             .accessibility_label("About")
                             .popover(showing_info.binding(),
                                      [] {
                                          return Text("Popovers close when you click elsewhere or press Esc.")
                                              .frame({.width = 200});
                                      },
                                      {.opens = PopoverEdge::trailing})),
                  Text([=] { return std::format("editing: {}", editing() ? "yes" : "no"); })
                      .font(Font::caption)
                      .foreground_style(ThemeColor::secondary))
        .padding(24)
        .frame({.max_width = infinity, .max_height = infinity});
}

} // namespace

int main() {
    App app;
    app.window({.title = "Popover", .width = 480, .height = 360}, PopoverApp);
    return app.run();
}
