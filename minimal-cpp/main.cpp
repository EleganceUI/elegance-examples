// Hello: the smallest Elegance app. One window with an icon and two lines of text.

#include <elg/elegance.h>

using namespace elg;

int main() {
    App app;
    app.window({.title = "Hello", .width = 480, .height = 320}, [] {
        return VStack({.spacing = 12},
                      Image("hand-waving").font(Font::system(48)).foreground_style(ThemeColor::accent),
                      Text("Hello, Elegance!").font(Font::title),
                      Text("A native UI in a few lines of C++.").foreground_style(ThemeColor::secondary));
    });
    return app.run();
}
