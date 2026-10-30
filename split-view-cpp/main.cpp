// NavigationSplitView: a sidebar list that selects what the detail column shows.

#include <elg/elegance.h>

#include <algorithm>
#include <format>
#include <string>
#include <vector>

using namespace elg;

namespace {

struct Game {
    int id = 0;
    std::string name;
    int mods = 0;

    friend bool operator==(const Game&, const Game&) = default;
};

const std::vector<Game> kGames{
    {1, "Minecraft", 42},
    {2, "Skyrim", 118},
    {3, "Stardew Valley", 9},
};

const Game* game_with(int id) {
    const auto it = std::ranges::find(kGames, id, &Game::id);
    return it == kGames.end() ? nullptr : &*it;
}

View SidebarRow(const Game& game, State<int> selected) {
    const int id = game.id;
    const State<bool> hover(false);
    return Text(game.name)
        .frame({.max_width = infinity, .alignment = Alignment::leading})
        .content_shape(Rectangle())
        .padding(EdgeInsets::symmetric(12, 10))
        .background([=](const EnvironmentValues& env) {
            return hover() ? env.color(ThemeColor::surface_raised) : Color::clear;
        })
        .on_hover_change([=](bool on) { hover = on; })
        .on_click([=] { selected = id; })
        .foreground_style([=](const EnvironmentValues& env) {
            return env.color(selected() == id ? ThemeColor::accent : ThemeColor::primary);
        });
}

View Sidebar(State<int> selected) {
    return ScrollView(
        LazyVStack({.spacing = 0}, ForEach(kGames, &Game::id, [=](const Game& game) { return SidebarRow(game, selected); })));
}

View Detail(State<int> selected, State<bool> sidebar_visible) {
    return VStack({.alignment = HorizontalAlignment::leading, .spacing = 8},
                  Text([=] {
                      const Game* game = game_with(selected());
                      return game ? game->name : std::string("Pick a game");
                  }).font(Font::title),
                  Text([=] {
                      const Game* game = game_with(selected());
                      return game ? std::format("{} mods installed", game->mods) : std::string();
                  }).foreground_style(ThemeColor::secondary),
                  Button("Toggle sidebar", [=] { sidebar_visible = !sidebar_visible(); }))
        .padding(24)
        .frame({.max_width = infinity, .max_height = infinity, .alignment = Alignment::top_leading});
}

View Library() {
    const State<int> selected(1);
    const State<bool> sidebar_visible(true);

    // Two views: sidebar, detail. (Three: sidebar, content, detail.)
    return NavigationSplitView(Sidebar(selected), Detail(selected, sidebar_visible),
                               {.sidebar_visible = sidebar_visible.binding(), .sidebar_width = {160, 200, 300}});
}

} // namespace

int main() {
    App app;
    app.window({.title = "Library", .width = 720, .height = 420}, Library);
    return app.run();
}
