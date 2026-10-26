// Fonts: the system font's named weights, any weight between them (variable fonts draw each one
// exactly), sizes, and italics.

#include <elg/elegance.h>

#include <format>
#include <string>
#include <utility>
#include <vector>

using namespace elg;

namespace {

constexpr const char* kPangram = "The quick brown fox jumps over the lazy dog";

View Section(std::string title, View content) {
    return VStack({.alignment = HorizontalAlignment::leading, .spacing = 12},
                  Text(std::move(title)).font(Font::headline), std::move(content));
}

std::string weight_name(int weight) {
    switch (weight) {
    case 100: return "ultra light";
    case 200: return "thin";
    case 300: return "light";
    case 400: return "regular";
    case 500: return "medium";
    case 600: return "semibold";
    case 700: return "bold";
    case 800: return "heavy";
    case 900: return "black";
    default: return "";
    }
}

// Each named weight, with the pangram and some figures and punctuation.
View Weights() {
    return Section("Weights",
                   VStack({.alignment = HorizontalAlignment::leading, .spacing = 6},
                          ForEach(std::vector<int>{100, 200, 300, 400, 500, 600, 700, 800, 900}, [](int weight) {
                              return HStack({.spacing = 16},
                                            Text(std::format("{} {}", weight, weight_name(weight)))
                                                .font(Font::caption)
                                                .foreground_style(ThemeColor::secondary)
                                                .frame({.width = 110, .alignment = Alignment::leading}),
                                            Text(std::format("{}  0123456789 &@?!", kPangram))
                                                .font(Font::system(20, static_cast<Weight>(weight))));
                          })));
}

// Any weight from 1 to 1000. (Each new weight's glyphs are generated once, then cached.)
View VariableWeight() {
    const State<float> weight(550.f);
    const auto font = [=] { return Font::system(44, static_cast<Weight>(static_cast<int>(weight()))); };
    return Section("Variable weight",
                   VStack({.alignment = HorizontalAlignment::leading, .spacing = 10},
                          // Text::font takes a fixed font; the environment's follows the slider.
                          View(Text("Elegance & Grace")).font(font),
                          HStack({.spacing = 12},
                                 Text([=] { return std::format("Weight {}", static_cast<int>(weight())); })
                                     .font(Font::system(13, Weight::medium))
                                     .frame({.width = 90, .alignment = Alignment::leading}),
                                 Slider(weight.binding(), {.min = 100, .max = 1000, .step = 25}))));
}

// A range of sizes, in regular and bold.
View Sizes() {
    return Section("Sizes",
                   HStack({.alignment = VerticalAlignment::bottom, .spacing = 22},
                          ForEach(std::vector<int>{11, 13, 15, 20, 28, 40, 64}, [](int size) {
                              return VStack({.spacing = 4},
                                            Text("Ag").font(Font::system(static_cast<float>(size))),
                                            Text("Ag").font(Font::system(static_cast<float>(size), Weight::bold)),
                                            Text(std::format("{} pt", size))
                                                .font(Font::caption2)
                                                .foreground_style(ThemeColor::secondary));
                          })));
}

View Italics() {
    return Section("Italics",
                   VStack({.alignment = HorizontalAlignment::leading, .spacing = 6},
                          Text(kPangram).font(Font::system(20).italic()),
                          Text(kPangram).font(Font::system(20, Weight::bold).italic())));
}

} // namespace

int main() {
    App app;
    app.window({.title = "Fonts", .width = 1000, .height = 860}, [] {
        return ScrollView(VStack({.alignment = HorizontalAlignment::leading, .spacing = 28}, Weights(),
                                 VariableWeight(), Sizes(), Italics())
                              .padding(24)
                              .frame({.max_width = infinity, .alignment = Alignment::leading}));
    });
    return app.run();
}
