# wasm-model-viewer-core

The Rust/WebAssembly engine behind [webgl-full-stack](https://github.com/stamps9k/webgl-full-stack). It parses Wavefront models (`.obj`, with `.mtl` materials and PNG textures), uploads them to the GPU and renders them with WebGL 2, with mouse and keyboard controls for rotating and moving around the model.

The compiled package is published to [wasm-model-viewer-core-bin](https://github.com/stamps9k/wasm-model-viewer-core-bin), which webgl-full-stack installs as the `wasm-model-viewer-core` npm package.

## Using it from JavaScript

The page needs a `<canvas id="glCanvas">`, and two global functions that the engine calls back into:

```js
// Receives the engine's log messages; level is a logger name such as "info_wasm_scene".
globalThis.wasm_logging_api = (message, level) => { /* ... */ };
// Receives the current frame rate.
globalThis.set_fps = (fps) => { /* ... */ };
```

Scene data is passed in as a `Map`:

| Key           | Value                                                            |
| ------------- | ---------------------------------------------------------------- |
| `vert_shader` | Vertex shader source (GLSL ES 3.0)                               |
| `frag_shader` | Fragment shader source                                           |
| `cube`        | The model's `.obj` text                                          |
| `materials`   | `Map` of material file name to `.mtl` text (optional)            |
| `textures`    | `Map` of texture file name to base64-encoded PNG data (optional) |

```js
import * as wasm from "wasm-model-viewer-core";

let engine = wasm.initialize_web_gl(resources); // sets up WebGL and the input listeners
engine = wasm.update_scene(engine, resources); // swap in a new model or shaders
```

The rotation and zoom toggles exposed to the UI are `enable_rotate_x` / `disable_rotate_x` (and the same for `y` and `z`), and `enable_zoom_in` / `disable_zoom_in` / `enable_zoom_out` / `disable_zoom_out`.

### Controls

- Drag with the left mouse button to rotate the model.
- Hold Shift and scroll to move the camera in and out.
- Hold Ctrl and scroll, or pinch on a trackpad, to pan.

## Building

Requires Rust with the `wasm32-unknown-unknown` target and `wasm-pack` (`cargo install wasm-pack --locked`).

```bash
wasm-pack build
```

This builds for bundlers (webgl-full-stack uses webpack) into `pkg/`. To publish, copy the contents of `pkg/` into the wasm-model-viewer-core-bin repo and push; that repo then triggers a rebuild of the webgl-full-stack image. `.cargo/config.toml` sets the `getrandom` WebAssembly backend that the `rand` crate needs.

## Development

```bash
cargo test
```

`cargo-husky` installs a pre-commit hook on the first `cargo test`. In a dev container that sets `CARGO_TARGET_DIR` to a volume, run `CARGO_TARGET_DIR=target cargo test` once so the hook can find the repo. The hook runs `cargo deny check advisories` when `Cargo.toml`, `Cargo.lock` or `deny.toml` change.

## License

GPL-3.0
