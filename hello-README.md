# hello-elegence

The same starter app in Rust and C++: a root component that shows a `HelloWorld` component with a
prop (`msg`) and a counter.

```
rust/src/main.rs                     cpp/src/main.cpp              opens the window
rust/src/root.rs                     cpp/src/root.cpp              the root component
rust/src/components/hello_world.rs   cpp/src/components/hello_world.{h,cpp}
```

## Rust

Needs Elegence's `dev-release` build (see `../elegence-rust/README.md`).

```sh
cd rust
cargo run
```

## C++

Builds Elegence along with the app (`add_subdirectory`); vcpkg reads Elegence's manifest. In a VS
developer prompt, or open `cpp/` in CLion with the `dev-debug` preset:

```sh
cd cpp
cmake --preset dev-debug
cmake --build build/dev-debug
build/dev-debug/bin/hello-elegence.exe
```

`CMakeUserPresets.json` points `VCPKG_ROOT` at `G:/elegence-vcpkg`; change it for another machine.
