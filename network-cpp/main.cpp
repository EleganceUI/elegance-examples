// Network: images loaded from the web (AsyncImage), with views of your own while one loads or if
// it fails, in a long, lazily built list. Downloads are cached on disk between runs.

#include <elg/elegance.h>

#include <format>
#include <vector>

using namespace elg;

namespace {

// A wide photo with its own view for each phase, and a button that forgets every download.
View Banner() {
    return HStack({.spacing = 12},
                  AsyncImage("https://picsum.photos/id/1015/480/160",
                             [](const AsyncImagePhase& phase) -> View {
                                 switch (phase.kind) {
                                 case AsyncImagePhase::Kind::success:
                                     return phase.image.resizable().aspect_ratio(ContentMode::fill);
                                 case AsyncImagePhase::Kind::failure:
                                     return Text(phase.error).foreground_style(ThemeColor::danger);
                                 case AsyncImagePhase::Kind::loading:
                                     break;
                                 }
                                 return Progress().frame({.width = 120});
                             })
                      .frame({.height = 80})
                      .frame({.max_width = infinity})
                      .clip_shape(RoundedRectangle(8)),
                  Button(Image("trash"), [] { clear_image_cache(); })
                      .button_style(BorderedButtonStyle{})
                      .accessibility_label("Clear the image cache"))
        .padding(16)
        .background(ThemeColor::surface, RoundedRectangle(12));
}

View Photo(int id) {
    return HStack({.spacing = 14},
                  AsyncImage(std::format("https://picsum.photos/id/{}/240/160", id),
                             {.content_mode = ContentMode::fill})
                      .frame(120, 80)
                      .clip_shape(RoundedRectangle(8)),
                  VStack({.alignment = HorizontalAlignment::leading, .spacing = 4},
                         Text(std::format("Photo #{}", id)).font(Font::headline),
                         Text("picsum.photos").foreground_style(ThemeColor::secondary)));
}

View Feed() {
    std::vector<int> ids;
    for (int id = 10; id < 210; ++id) {
        ids.push_back(id);
    }
    return VStack({.alignment = HorizontalAlignment::leading, .spacing = 16},
                  Banner(),
                  // Only the rows near the visible part are built, so only their images load.
                  ScrollView(LazyVStack({.alignment = HorizontalAlignment::leading,
                                         .spacing = 12,
                                         .separators = StackSeparators{.inset = 134}},
                                        ForEach(ids, [](int id) { return Photo(id); }))))
        .padding(20);
}

} // namespace

int main() {
    App app;
    app.window({.title = "Network", .width = 560, .height = 640}, Feed);
    return app.run();
}
