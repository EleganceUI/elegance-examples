# Elegance examples

Example apps for [Elegance](https://github.com/EleganceUI/elegance), in C++ (`<name>-cpp`) and in
Rust with [elegance-rs](https://github.com/EleganceUI/elegance-rs) (`<name>-rs`).

Every folder is a complete project with its own build files: **copy one to start an app.** `hello`
is the starter (a root component and a component with a prop and state); `preview` sets up hot
reload and previews.

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| hello | [✓](hello-cpp) | [✓](hello-rs) | A starter app: a root component, and a component with a prop and state |
| preview | [✓](preview-cpp) | [✓](preview-rs) | Hot reload and previews: views that rebuild and reload as you edit |
| minimal | [✓](minimal-cpp) | | The smallest app: one window, an icon and two lines of text |
| counter | [✓](counter-cpp) | [✓](counter-rs) | State and reactive arguments |
| todos | [✓](todos-cpp) | [✓](todos-rs) | A list in state: ForEach, If, a computed count, animated insertions |
| components | | [✓](components-rs) | Custom components and every kind of prop: defaults, Option, reactive, two-way, callbacks, children, leading-dot enums, a modifier with named arguments |
| layout | [✓](layout-cpp) | | Stacks, spacers, frames, overlays, and a custom flow layout |
| styling | [✓](styling-cpp) | | A custom theme, light and dark mode, a button style, a view modifier |
| drawing | [✓](drawing-cpp) | | Gradients, strokes, an SVG path, a computed shape, a Canvas chart |
| animation | [✓](animation-cpp) | | Transitions, an implicit spring, tiles that slide to new places |
| spinner | [✓](spinner-cpp) | | A loading spinner: on_appear and a repeating animation |
| effects | [✓](effects-cpp) | [✓](effects-rs) | Frosted glass (backdrop blur), blur, color adjustments, blend modes, shadows, masks |
| fonts | [✓](fonts-cpp) | | The system font's weights, any weight between them (variable fonts), sizes, italics |
| grids | [✓](grids-cpp) | [✓](grids-rs) | An adaptive VGrid of cards, and a Grid table |
| popover | [✓](popover-cpp) | [✓](popover-rs) | Panels shown beside a view while a binding is true |
| navigation | [✓](navigation-cpp) | | NavigationSplitView, NavigationStack, VSplitView |
| split-view | | [✓](split-view-rs) | A sidebar list that selects the detail |
| icon-strip | | [✓](icon-strip-rs) | A strip of icons down the side that selects the page |
| title-bar | [✓](title-bar-cpp) | | A window with its own title bar and a translucent, rounded panel |
| media-player | [✓](media-player-cpp) | | A video player: open or drop a file |
| network | [✓](network-cpp) | | Web images with custom loading and failure views, in a lazy list, cached on disk |
| recorder | [✓](recorder-cpp) | | Voice memos with a level meter, and a camera preview |
| desktop | [✓](desktop-cpp) | | File dialogs, notifications, a tray icon, dropped files |
| wgpu-image | | [✓](wgpu-image-rs) | A wgpu cube read back through CPU memory into a VideoFeed |
| wgpu-direct | | [✓](wgpu-direct-rs) | The same cube rendered directly into a shared GPU image, without CPU pixel readback |
| login | [✓](login-cpp) | | A modern sign-in screen with a QR code |
| music | [✓](music-cpp) | | A desktop music player |
| shell | [✓](shell-cpp) | | A desktop environment: windows, a dock, a launcher, sample apps |
| design | [✓](design-cpp) | | A Figma-style editor on an infinite canvas |

## C++

You need Visual Studio's C++ workload (or Apple Clang on macOS), CMake 4.3+, Ninja, and
[vcpkg](https://vcpkg.io) with `VCPKG_ROOT` set. In a developer prompt:

```sh
cd counter-cpp
cmake --preset windows-release      # or windows-debug, macos-debug, macos-release
cmake --build --preset windows-release
build\windows-release\bin\counter.exe
```

Each project builds Elegance along with the app. It uses `ELEGANCE_DIR` if set, else a checkout of
[elegance](https://github.com/EleganceUI/elegance) beside this repository (`../elegance`), and
otherwise downloads it. vcpkg reads only the app's own `vcpkg.json`, so each one lists Elegance's
dependencies (keep them in step with Elegance's `vcpkg.json` when you update it). The first
configure builds them, Dawn included: allow about half an hour.

The `CMakeLists.txt` here builds every C++ example at once, with one Elegance:
`cmake --preset windows-release` in this folder.

## Rust

You need Rust 1.88+, Visual Studio's C++ workload (or Apple Clang), and git.

```sh
cd counter-rs
cargo run
```

`elegance-rs` builds Elegance the first time (it takes a while; `cargo build -vv` shows progress),
finding or setting up vcpkg, CMake and Ninja itself. See its README for the details.

The `Cargo.toml` here makes every Rust example one workspace, so they share a build:
`cargo build --workspace`. This workspace patches Elegance to the sibling `../elegance-rs`
checkout so the direct-GPU bridge and the examples use the same bindings. In another
workspace, the equivalent local override is:

```toml
[patch."https://github.com/EleganceUI/elegance-rs"]
elegance = { path = "../elegance-rs/elegance" }
```

### wgpu cube examples

The cube examples keep the same drag-to-rotate, scroll-to-zoom and speed controls:

```sh
cargo run -p wgpu-image
cargo run -p wgpu-direct
```

`wgpu-image-rs` retains the cross-platform CPU-readback/VideoFeed path.
`wgpu-direct-rs` uses the sibling `elegance-wgpu` crate and its pinned wgpu re-export,
with no C++ or backend-specific application code. It requires Rust 1.93+ and compatible
GPU-sharing support; Metal is hardware-tested, while DX12 and Vulkan implementations
still need hardware validation. D3D11 and OpenGL/GLES sharing are not implemented.
Unsupported sharing returns an error, not a CPU fallback. See its README for details.
Both producers use a nominal 60 Hz throttle, not a display-VSync callback; Elegance's
window presentation remains FIFO/VSync.

### Previews

```sh
cargo install --git https://github.com/EleganceUI/elegance-rs cargo-elegance
cd preview-rs
cargo elegance preview
```

## Starting your own app

Copy a folder (`hello-cpp` or `hello-rs`, say) wherever you like, and rename the project:
`project()`, the target, and `"name"` in `vcpkg.json` for C++; `name` in `Cargo.toml` for Rust.

## License

[MIT](LICENSE), so you can copy the examples into your own apps freely. (Elegance itself is
MPL-2.0.)
