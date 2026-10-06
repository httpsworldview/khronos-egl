# Rust bindings for EGL

This crate provides a binding for the Khronos EGL 1.5 API.

## Project status

This repository is a fork of
[timothee-haudebourg/khronos-egl](https://github.com/timothee-haudebourg/khronos-egl),
which I plan on maintaining independently.

If you want to use this library, see [CHANGELOG.md](CHANGELOG.md) and
refer to source level documentation.

## Usage

Enable `static` for link-time EGL binding or `dynamic` for runtime
loading. See the [crate-level guide](src/lib.rs) for setup, version
selection, examples, and troubleshooting.

To read the rendered guide locally:

```sh
cargo doc --no-deps --features "static dynamic" --open
```

Runnable examples:
- [Static Wayland application](examples/wayland-static.rs)
- [Dynamic Wayland application](examples/wayland-dynamic.rs)
- [Minimal dynamic loading and version selection](examples/load-minimal.rs)

### NixOS

A `shell.nix` file is present for nix users to build the crate easily.
Just enter a new nix shell using the given configuration file, and
`cargo build` should work.  If you want to run the tests and examples
you will need to use `shell-wayland.nix` instead that will also load
wayland since most of them depend on it.

## Testing

Install the EGL and Wayland development libraries, then run:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --features "static dynamic" -- -D warnings
cargo test
cargo test --features "static dynamic"
cargo run --example load-minimal --no-default-features --features "dynamic 1_4"
```

Doctests that need a running Wayland compositor are compiled but not
executed.  The Wayland examples require a compositor to run.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

If the original `egl` crate was licensed only under the Apache 2.0
license, I believe I have made enough breaking changes so that no
relevant code from the original code remains and the rest can be
relicensed.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
