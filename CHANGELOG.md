Changelog
=========

## [unreleased]

## v0.2.0
- Update dependencies
- Migrated the repo to Codeberg with Woodpecker CI.
- Fix bug where the generated javascript to call init didn't work for
  wasm-bindgen 0.2.85 and newer.
- Simplify the cargo:rerun-if-changed paths printed by making use of cargo
  1.50's ability to watch whole directories.

## v0.1.4
- Fix bug where environment variables set for the workspace build
  would leak into build scripts in the Wasm build.

## v0.1.3
- Remove requirement to have `wasm-pack` CLI installed and on the path.
