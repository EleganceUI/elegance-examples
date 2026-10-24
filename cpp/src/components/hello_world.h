#pragma once

#include <elg/view.h>

#include <string>

// A component with a prop (`msg`) and some state of its own (a counter). Any type with
// `View body() const` can be used as a view; body() runs when the component is mounted.
struct HelloWorld {
    std::string msg;

    elg::View body() const;
};
