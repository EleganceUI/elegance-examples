// The entry point: opens a window showing the root component.

#include <elg/elegance.h>

#include "root.h"

int main() {
    elg::App app;
    app.window({.title = "Hello Elegance", .width = 640, .height = 420}, Root);
    return app.run();
}
