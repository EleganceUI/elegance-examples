// Preview: hot reload. The views live in a module (views/), a shared library; this app shows it,
// and when a file in views/ changes it rebuilds the module and reloads it in place. Run it, edit
// views/profile_card.cpp, save, and watch the window update.
//
//   preview              the app's root view (Root in views/root.cpp)
//   preview --previews   the module's previews (ELG_PREVIEW), listed beside the selected one

#include "preview_hot_reload.h"

#include <elg/elegance.h>

#include <string_view>

int main(int argc, char** argv) {
    elg::HotReloadOptions options = elg_hot_reload_options();
    if (argc > 1 && std::string_view(argv[1]) == "--previews") {
        options.show = elg::HotReloadShow::previews;
    }

    elg::App app;
    app.window({.title = "Preview", .width = 760, .height = 560}, [=] { return elg::HotReload(options); });
    return app.run();
}
