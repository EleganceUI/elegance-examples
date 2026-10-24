// Counter: state and reactive arguments. Writing `count` updates exactly the views whose
// closures read it; nothing else is rebuilt.

#include <elg/elegance.h>

#include <format>
#include <iostream>

using namespace elg;

View Counter() {
    const State<int> count(0);

    return VStack({.spacing = 16},
                  Text([=] { return std::format("{}", count()); }).font(Font::system(56, Weight::bold)),
                  HStack(Button("−", [=] { count -= 1; }),
                         Button("Reset", [=] { count = 0; })
                             .button_style(BorderedButtonStyle{})
                             .disabled([=] { return count() == 0; }),
                         Button("+", [=] { count += 1; })),
                  Text([=] { return count() % 2 == 0 ? "even" : "odd"; })
                      .foreground_style(ThemeColor::secondary));
}

int main() {
    App app;

    app.window({.title = "Counter", .width = 360, .height = 280}, Counter);
    auto res = app.run();

    return res;
}
