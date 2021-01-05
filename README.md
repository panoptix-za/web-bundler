Web Bundler
===========

Manages the building of WebAssembly single page app frontends, so that
they can be embedded into Rust api projects.

The process is exposed via a library, which can be called from a
build.rs script.

Internally, the bundler calls
[wasm-pack](https://github.com/rustwasm/wasm-pack) to do the actual
webassembly compilation, which must be available on the path.

## Prerequisites

From a clean Rustup-based Rust install, you'd need to add these steps:

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-pack`

## Additional nice features to add one day:

- Read the Cargo.toml from the webassembly crate, rather than passing
  it in as an argument.
- Automatically find the root of the workspace from the current crate,
  rather than passing it in as an argument.
- Figure out the Sass extra compile dirs automatically, or declare the
  dependency in Cargo.toml somehow.
- Figure out the dependency chain to log for the Rust crate from
  web-target/wasm32-unknown-unknown/release/hotrod_admin.d, so that
  the api project doesn't need to declare it as an unused build
  dependency.
- Autoprefixer support
- A cli frontend for rebuilding the web frontend without needing to
  recompile the API.
