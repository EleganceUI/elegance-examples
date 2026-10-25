// Preview: hot reload. The views live in a module (views/), a shared library;
// this app shows it, and when a file in views/ changes it rebuilds the module
// and reloads it in place. Run it, edit views/profile_card.cpp, save, and watch
// the window update. Static release builds link these views into the executable
// instead, without hot reload.
//
//   preview              the app's root view (Root in views/root.cpp)
//   preview --previews   the module's previews (shared debug builds only)

#include <elg/elegance.h>

#include <cstdio>
#include <string_view>

#if ELG_PREVIEW_HOT_RELOAD
#include "preview_hot_reload.h"
#else
extern "C" const elg::ModuleEntry *elg_module_entry();
#endif

int main(int argc, char **argv) {
  elg::App app;
#if ELG_PREVIEW_HOT_RELOAD
  elg::HotReloadOptions options = elg_hot_reload_options();
  if (argc > 1 && std::string_view(argv[1]) == "--previews") {
    options.show = elg::HotReloadShow::previews;
  }

  app.window({.title = "Preview", .width = 760, .height = 560},
             [=] { return elg::HotReload(options); });
#else
  if (argc > 1 && std::string_view(argv[1]) == "--previews") {
    std::fputs("--previews requires the shared debug build.\n", stderr);
    return 1;
  }
  app.window({.title = "Preview", .width = 760, .height = 560},
             [] { return elg_module_entry()->root(); });
#endif
  return app.run();
}
