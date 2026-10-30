// Spinner: the orbiting dots loader from wpdean.com/css-animation. Three dots chase each other
// around a faint ring, 1.4 s a turn. They follow the same eased path, each 0.12 s ahead of the one
// before, so they bunch up where it's slow and spread out where it's fast.
//
// on_appear starts the turn, and a repeating animation keeps it going for as long as the loader
// is shown. The CSS it's made from:
//   animation: an-orbit 1.4s cubic-bezier(.55, .1, .45, .9) infinite;
//   animation-delay: -.12s;  (the second dot; -.24s for the third)

#include <elg/elegance.h>

using namespace elg;

namespace {

View OrbitingDots() {
    const State<bool> spinning(false);
    const auto dot = [=](Color color, double ahead) {
        return Circle()
            .fill(color)
            .frame(12, 12)
            .offset(0, -6) // its middle on the ring
            .frame(72, 72, Alignment::top)
            .rotation_effect([=] { return spinning() ? 360.f : 0.f; })
            .animation(Animation::timing_curve(0.55, 0.1, 0.45, 0.9, 1.4).delay(-ahead).repeat_forever(false), spinning);
    };
    return ZStack(Circle().stroke(Color::white.opacity(0.12f), 2),
                  dot(Color::hex(0xFF6B3D), 0),
                  dot(Color::hex(0xFFD84D), 0.12),
                  dot(Color::hex(0x7EA0FF), 0.24))
        .frame(72, 72)
        .accessibility_label("Loading")
        .on_appear([=] { spinning = true; });
}

} // namespace

int main() {
    App app;
    app.window({.title = "Spinner", .width = 240, .height = 240}, [] {
        return OrbitingDots()
            .frame({.max_width = infinity, .max_height = infinity})
            .background(Color::hex(0x16161D));
    });
    return app.run();
}
