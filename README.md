<h1 align="center">Elegance Examples</h1>

<p align="center">
  <strong>Example apps for <a href="https://github.com/EleganceUI/elegance">Elegance</a>, in C++ and Rust.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-green.svg"></a>
</p>

<p align="center">
  <a href="https://eleganceui.github.io/elegance-docs/">Documentation</a> ·
  <a href="https://github.com/EleganceUI/elegance">Elegance (C++)</a> ·
  <a href="https://github.com/EleganceUI/elegance-rs">elegance-rs (Rust)</a>
</p>

---

Every folder is a complete, standalone project with its own build files: **copy one to start
your own app.** C++ projects end in `-cpp`; Rust projects, built on
[elegance-rs](https://github.com/EleganceUI/elegance-rs), end in `-rs`.

New to Elegance? Start with **hello**, a root component plus a component with a prop and state.
Then try **preview** for hot reload.

## Catalog

### Basics

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| hello | [✓](hello-cpp) | [✓](hello-rs) | A root component, and a component with a prop and state |
| minimal | [✓](minimal-cpp) | | The smallest app: one window, an icon, and two lines of text |
| counter | [✓](counter-cpp) | [✓](counter-rs) | State and reactive arguments |
| todos | [✓](todos-cpp) | [✓](todos-rs) | A list in state: `ForEach`, `If`, a computed count, animated insertions |
| components | | [✓](components-rs) | Custom components and every kind of prop: defaults, `Option`, reactive, two-way, callbacks, children, leading-dot enums, named-argument modifiers |
| preview | [✓](preview-cpp) | [✓](preview-rs) | Hot reload and previews |

### Layout and navigation

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| layout | [✓](layout-cpp) | | Stacks, spacers, frames, overlays, and a custom flow layout |
| grids | [✓](grids-cpp) | [✓](grids-rs) | An adaptive `VGrid` of cards, and a `Grid` table |
| navigation | [✓](navigation-cpp) | | `NavigationSplitView`, `NavigationStack`, `VSplitView` |
| split-view | | [✓](split-view-rs) | A sidebar list that selects the detail view |
| icon-strip | | [✓](icon-strip-rs) | A vertical strip of icons that selects the page |
| popover | [✓](popover-cpp) | [✓](popover-rs) | Panels shown beside a view while a binding is true |
| title-bar | [✓](title-bar-cpp) | | A custom title bar and a translucent, rounded window |

### Styling and graphics

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| styling | [✓](styling-cpp) | | A custom theme, light and dark mode, a button style, a view modifier |
| fonts | [✓](fonts-cpp) | | System font weights, variable font weights, sizes, italics |
| drawing | [✓](drawing-cpp) | | Gradients, strokes, an SVG path, a computed shape, a `Canvas` chart |
| effects | [✓](effects-cpp) | [✓](effects-rs) | Backdrop blur, blur, color adjustments, blend modes, shadows, masks |
| animation | [✓](animation-cpp) | | Transitions, an implicit spring, tiles that slide into new positions |
| spinner | [✓](spinner-cpp) | | A loading spinner: `on_appear` and a repeating animation |
| wgpu-image | | [✓](wgpu-image-rs) | A wgpu cube read back through CPU memory into a `VideoFeed` |
| wgpu-direct | | [✓](wgpu-direct-rs) | The same cube rendered straight into a shared GPU image, with no CPU readback |

### Media and desktop

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| media-player | [✓](media-player-cpp) | | A video player: open or drop a file |
| recorder | [✓](recorder-cpp) | | Voice memos with a level meter, and a camera preview |
| network | [✓](network-cpp) | | Web images in a lazy list, with loading and failure views and a disk cache |
| desktop | [✓](desktop-cpp) | | File dialogs, notifications, a tray icon, dropped files |

### Showcase apps

| Example | C++ | Rust | What it shows |
| --- | :-: | :-: | --- |
| login | [✓](login-cpp) | | A modern sign-in screen with a QR code |
| music | [✓](music-cpp) | | A desktop music player |
| shell | [✓](shell-cpp) | | A desktop environment: windows, a dock, a launcher, sample apps |
| design | [✓](design-cpp) | | A Figma-style editor on an infinite canvas |

## Running the Rust examples

You need Rust 1.88 or newer. No C++ toolchain is required: elegance-rs downloads a prebuilt,
statically linked Elegance.

```sh
cd counter-rs
cargo run
```

The `Cargo.toml` at the root makes every Rust example one workspace with a shared build:

```sh
cargo build --workspace
cargo run -p counter
```

On Linux, add `$ORIGIN` to the rpath so executables find the shared libraries Cargo copies beside
them (see [elegance-rs's README](https://github.com/EleganceUI/elegance-rs#linking-and-distribution)).

For previews and hot reload:

```sh
cargo install --git https://github.com/EleganceUI/elegance-rs cargo-elegance
cd preview-rs
cargo elegance preview
```

### wgpu cube examples

Both cube examples support drag to rotate, scroll to zoom, and a speed control:

```sh
cargo run -p wgpu-image     # portable: CPU readback into a VideoFeed
cargo run -p wgpu-direct    # shared GPU memory through elegance-wgpu, no CPU readback
```

`wgpu-direct` needs Rust 1.93 or newer. Its GPU sharing is tested on hardware with Metal. The
Direct3D 12 and Vulkan paths are implemented but not yet validated on hardware, and Direct3D 11
and OpenGL aren't supported. Unsupported configurations report an error rather than falling back
to CPU copies. Both producers render at a nominal 60 Hz, while Elegance presents windows with
VSync.

## Running the C++ examples

You need CMake 4.3+, Ninja, [vcpkg](https://vcpkg.io) with `VCPKG_ROOT` set, and a compiler:
Visual Studio's C++ workload on Windows, Apple Clang from Xcode 26+ on macOS, or GCC 14+ on Linux.

```sh
cd counter-cpp
cmake --preset windows-release      # or windows-debug, macos-debug, macos-release
cmake --build --preset windows-release
build/windows-release/bin/counter
```

Each project builds Elegance along with the app, taking it from `ELEGANCE_DIR` if set, otherwise
from an [elegance](https://github.com/EleganceUI/elegance) checkout beside this repository
(`../elegance`), and otherwise downloading it. The first configure builds Elegance's
dependencies, Dawn included, which takes about half an hour.

vcpkg reads only the app's own `vcpkg.json`, so each project lists Elegance's dependencies. Keep
them in step with Elegance's `vcpkg.json` when you update it.

The `CMakeLists.txt` at the root builds every C++ example against one Elegance:
`cmake --preset windows-release` in this folder.

## Starting your own app

1. Copy a folder (`hello-cpp` or `hello-rs`, say) wherever you like.
2. Rename the project:
   - **C++:** `project()` and the target in `CMakeLists.txt`, and `"name"` in `vcpkg.json`
   - **Rust:** `name` in `Cargo.toml`

## Developing against local checkouts

To test changes to elegance-rs, create `.cargo/config.toml` in this folder:

```toml
[patch."https://github.com/EleganceUI/elegance-rs"]
elegance = { path = "../elegance-rs/elegance" }
```

C++ projects pick up a checkout at `../elegance` automatically.

## License

The examples are licensed under [MIT](LICENSE), so you can copy them into your own apps freely.
Elegance and elegance-rs are licensed under MPL-2.0.
