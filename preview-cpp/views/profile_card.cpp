#include "profile_card.h"

#include <elg/elegance.h>

using namespace elg;

namespace {

std::string Initials(const std::string& name) {
    std::string initials;
    for (std::size_t i = 0; i < name.size(); ++i) {
        if (i == 0 || name[i - 1] == ' ') {
            initials += name[i];
        }
    }
    return initials;
}

} // namespace

View ProfileCard(std::string name, std::string role) {
    // Before the call below: it moves `name`, and arguments are evaluated in no particular order.
    std::string initials = Initials(name);
    return HStack({.spacing = 14},
                  Circle()
                      .fill(ThemeColor::accent)
                      .frame(48, 48)
                      .overlay(Text(std::move(initials)).font(Font::headline).foreground_style(Color::white)),
                  VStack({.alignment = HorizontalAlignment::leading, .spacing = 2},
                         Text(std::move(name)).font(Font::headline),
                         Text(std::move(role)).foreground_style(ThemeColor::secondary)),
                  Spacer())
        .padding(16)
        .frame({.width = 320})
        .background(RoundedRectangle(14).fill(ThemeColor::surface));
}

// Previews: the card on its own, with sample data. `preview --previews` lists them.

ELG_PREVIEW("Profile card") {
    return ProfileCard("Ada Lovelace", "Analyst").padding(24);
}

ELG_PREVIEW("Profile card, dark") {
    return ProfileCard("Grace Hopper", "Rear admiral")
        .padding(24)
        .background(ThemeColor::background)
        .color_scheme(ColorScheme::dark);
}
