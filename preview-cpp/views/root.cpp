// The views module's root view, which the app shows. State made here starts over on each reload;
// keep data that should survive in the app and pass it in.

#include <elg/elegance.h>

#include <format>

#include "profile_card.h"

using namespace elg;

namespace {

View Root() {
    const State<int> clicks(0);
    return VStack({.alignment = HorizontalAlignment::leading, .spacing = 14},
                  Text("Team").font(Font::large_title),
                  Text("Edit views/profile_card.cpp and save.").foreground_style(ThemeColor::secondary),
                  ProfileCard("Ada Lovelace", "Analyst"), ProfileCard("Grace Hopper", "Rear admiral"),
                  Button(Text([=] { return std::format("Clicked {} times", clicks()); }),
                         [=] { clicks += 1; }))
        .padding(32)
        .frame({.max_width = infinity, .max_height = infinity, .alignment = Alignment::top_leading});
}

} // namespace

ELG_MODULE(Root)

ELG_PREVIEW("Whole screen") {
    return Root();
}
