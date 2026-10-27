// Network: HTTP with an established library (cpp-httplib) on a background thread, its result
// handed to the UI with post_to_ui_thread, and images loaded from the web in a long, lazily built
// list (AsyncImage).

#include <elg/elegance.h>

#include <httplib.h>

#include <expected>
#include <format>
#include <thread>
#include <vector>

using namespace elg;

namespace {

using Body = std::expected<std::string, std::string>; // the body, or why there isn't one

// GETs `path` from `server` on a thread of its own; `done` runs on the UI thread, where it may
// write state.
void get(std::string server, std::string path, std::function<void(const Body&)> done) {
    std::thread([server = std::move(server), path = std::move(path), done = std::move(done)] {
        httplib::Client client(server);
        client.set_follow_location(true);
        client.set_connection_timeout(10);
        const auto response = client.Get(path, httplib::Headers{{"User-Agent", "elegance-example"}});
        Body body;
        if (!response) {
            body = std::unexpected(httplib::to_string(response.error())); // network, TLS, timeout
        } else if (response->status != 200) {
            body = std::unexpected(std::format("HTTP {}", response->status));
        } else {
            body = response->body;
        }
        post_to_ui_thread([done, body = std::move(body)] { done(body); });
    }).detach();
}

// A quote from GitHub's API, with a button to fetch another.
View Quote() {
    const State<bool> loading(true);
    const State<std::string> quote(std::string{});
    const auto load = [=] {
        loading = true;
        get("https://api.github.com", "/zen", [=](const Body& body) {
            quote = body ? *body : body.error();
            loading = false;
        });
    };
    load();
    return HStack({.spacing = 12},
                  Image("quotes").font(Font::title2).foreground_style(ThemeColor::accent),
                  If([=] { return loading() && quote().empty(); },
                     Progress().frame({.width = 120}),
                     Text([=] { return quote(); }).font(Font::title3)),
                  Spacer(),
                  Button(Image("arrow-clockwise"), load)
                      .button_style(BorderedButtonStyle{})
                      .disabled([=] { return loading(); })
                      .accessibility_label("Another quote"))
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
                  Quote(),
                  // Only the rows near the visible part are built, so only their images load.
                  ScrollView(LazyVStack({.spacing = 0, .separators = StackSeparators{.inset = 16}},
                                        ForEach(ids,
                                                [](int id) {
                                                    return Photo(id)
                                                        .padding(EdgeInsets::symmetric(16, 10))
                                                        .frame({.max_width = infinity, .alignment = Alignment::leading});
                                                }))
                                 .background(ThemeColor::surface, RoundedRectangle(10))))
        .padding(20);
}

} // namespace

int main() {
    App app;
    app.window({.title = "Network", .width = 560, .height = 640}, Feed);
    return app.run();
}
