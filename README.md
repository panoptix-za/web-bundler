Web Bundler
===========

Manages the building of WebAssembly single page app frontends from a
build.rs script so that they can easily be embedded into Rust api
projects.

Internally, the bundler calls the
[wasm-pack](https://github.com/rustwasm/wasm-pack) CLI to do the
actual webassembly compilation, which must be installed and available
on the path.

## Prerequisites

From a clean Rustup-based Rust install, you'd need to add these steps:

- `rustup target add wasm32-unknown-unknown`
- `cargo install wasm-pack`

## Running the Demo Example

There is an example usage in the [example directory](./example). To
run the example, open a terminal in the example directory and run
`cargo run`. Then, open a web browser and navigate to
<http://localhost:3030/>. You should see a Seed web application.

## Usage

Web-bundler expects you to have two projects: a frontend project using
a single page app framework like Seed, and a backend project using a
web server framework like Warp. These projects should be in a common
workspace.

### Changes to your frontend project

1. Update your index.html to allow templating in Javascript and CSS.

Specifically, you need to add `{{ stylesheet | safe }}` to the
`<head>` section, and `{{ javascript | safe }}` to the end of the
`<body>`. Optionally, if you want to set the base url, add `<base
href="{{ base_url }}">` to the `<head>` as well.

See the example [frontend index.html](./example/frontend/index.html).

2. Create a root stylesheet for your app called `./css/style.scss`.

This stylesheet will be compiled to CSS, and embedded directly into
your index.html file.

See the example [frontend style.scss](./example/frontend/css/style.scss).

3. Put all of your static assets in the `static` directory

All files in the static directory will be copied directly to a static
folder in the output directory.

See the example [frontend static directory](./example/frontend/static/).

### Changes to your API project

1. Update your Cargo.toml to depend on your frontend project and web-bundler

We depend on the frontend project in Cargo.toml so that Cargo knows to
rerun build.rs whenever the frontend project changes.

See the example [backend Cargo.toml](./example/backend/Cargo.toml).

2. Add a build.rs script that calls web-bundler for your frontend

See the example [backend build.rs](./example/backend/build.rs).

3. Use [Rust Embed](https://lib.rs/crates/rust-embed) to embed your built frontend into your API binary

See the example [backend main.rs](./example/backend/main.rs). Our
example uses the Warp web server. Rust Embed also has examples for
other web servers in [their repo](https://github.com/pyros2097/rust-embed/tree/master/examples).

## target and web-target directories

When web-bundler compiles the frontend, it overrides the default
target directory to be `web-target` instead of `target`. This is done
because, if the backend and frontend are in the same workspace, Cargo
will already be locking `target` while running the build.rs
script.
