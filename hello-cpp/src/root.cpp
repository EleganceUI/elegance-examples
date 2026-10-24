#include "root.h"

#include <elg/elegence.h>

#include "components/hello_world.h"

using namespace elg;

View Root() {
    return VStack({.spacing = 24},
                  Image("rocket-launch").font(Font::system(64)).foreground_style(ThemeColor::accent),
                  HelloWorld{.msg = "You did it!"})
        .padding(32)
        .frame({.max_width = infinity, .max_height = infinity});
}
