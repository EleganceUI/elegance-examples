#include "components/hello_world.h"

#include <elg/elegence.h>

#include <format>

using namespace elg;

View HelloWorld::body() const {
    const State<int> count(0);

    return VStack({.spacing = 16},
                  Text(msg).font(Font::large_title),
                  Button(Text([=] { return std::format("count is {}", count()); }), [=] { count += 1; }),
                  Text("Edit src/components/hello_world.cpp and rebuild to see changes.")
                      .foreground_style(ThemeColor::secondary));
}
