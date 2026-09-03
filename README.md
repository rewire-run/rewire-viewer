<h1 align="center">
  <a href="https://rewire.run/">
    <img alt="rewire" src="https://rewire.run/brand/rewire-banner.png">
  </a>
</h1>

<p align="center">
  <a href="https://github.com/rewire-run/viewer/actions/workflows/ci.yaml">
    <img alt="CI" src="https://github.com/rewire-run/viewer/actions/workflows/ci.yaml/badge.svg">
  </a>
  <img alt="Version" src="https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Frewire-run%2Fviewer%2Fmain%2FCargo.toml&query=%24.package.version&prefix=v&label=version&color=green">
  <a href="https://github.com/rewire-run/viewer/blob/main/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue">
  </a>
  <a href="https://pixi.sh">
    <img alt="Powered by" src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json">
  </a>
</p>

A custom [Rerun](https://rerun.io) viewer for ROS 2 visualization, built on top of Rerun v0.37.

## Features

- **Topics Panel** — sortable table of subscribed ROS 2 topics with type, publisher count, and subscriber count
- **Nodes Panel** — sortable table of discovered ROS 2 nodes with transport info
- **Status Bar** — live connection state, relay-sourced bridge count, node count, topic count, and uptime
- **Self-healing connection** — dials a relay (or any Rerun gRPC endpoint) and auto-reconnects with
  backoff; start order does not matter

## Build

Requires Rust 1.82+.

```bash
cargo build --release
```

Or with [pixi](https://pixi.sh):

```bash
pixi run build
pixi run sanity   # check + fmt + lint + test
```

## Run

```bash
cargo run --release
```

By default the viewer connects to `rerun+http://127.0.0.1:9876/proxy`. Point it elsewhere with
`--connect`, which accepts `host`, `host:port`, or a full proxy URL:

```bash
rewire-viewer --connect robot.local:9876
```

If no relay is running yet, the viewer waits in "Connecting..." and attaches as soon as one
appears; if the relay restarts, it reconnects on its own.

## Web viewer

The WebAssembly build (`app.rewire.run`) reads two query parameters:

| Parameter | Values | Default |
|-----------|--------|---------|
| `url` | a recording to open on start | — |
| `theme` | `dark` · `light` · `system` | follows the browser |

```
https://app.rewire.run/?url=https://examples.rewire.run/turtlebot3-example.rrd&theme=dark
```

### Theming an embed

Without `theme`, the viewer follows the browser's `prefers-color-scheme`. A page embedding the
viewer in an iframe **cannot** override that from the outside: `color-scheme` does not cross an
origin boundary, and the UI is painted into a canvas rather than out of browser widgets, so there
is nothing for the embedder's CSS to restyle. `theme` is the supported way to pin an embed.

To follow a host page's own theme toggle *without* reloading the iframe — a reload would
re-download the recording — post a message to it:

```js
frame.contentWindow.postMessage({ type: "rewire:theme", theme: "light" }, "*");
```

The viewer applies `dark`, `light` and `system` and ignores anything else. The message only
changes appearance, so it is accepted from any origin.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
