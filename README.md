# Rewire Viewer

A custom [Rerun](https://rerun.io) viewer for ROS 2 visualization, built on top of Rerun v0.30.

## Features

- **Topics Panel** — sortable table of subscribed ROS 2 topics with type, publisher count, and subscriber count
- **Nodes Panel** — sortable table of discovered ROS 2 nodes with transport info
- **Status Bar** — real-time connection status, bridge count, node count, topic count, and uptime
- **HTTP API** — info and heartbeat endpoints for bridge integration

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

The viewer starts two servers:

| Port | Protocol | Purpose                              |
|------|----------|--------------------------------------|
| 9876 | gRPC     | Rerun data stream (connect with `--connect 127.0.0.1:9876`) |
| 9877 | HTTP     | Viewer API (`GET /api/info`, `POST /api/heartbeat/{id}`)    |

## Architecture

```
src/
  main.rs              — entry point
  app.rs               — RewireApp (eframe wrapper with status bar)
  http.rs              — HTTP API server (axum)
  util.rs              — shared utilities
  ui/
    status_bar.rs      — connection status bar widget
  views/
    topics/            — Topics SpaceView (ViewClass + VisualizerSystem)
    nodes/             — Nodes SpaceView (ViewClass + VisualizerSystem)
```

## Dependencies

- [rerun](https://github.com/rerun-io/rerun) v0.30 — visualization framework
- [rewire-extras](https://github.com/rewire-run/rewire-extras) — shared ROS 2 archetypes
- [axum](https://github.com/tokio-rs/axum) — HTTP server
- [egui_extras](https://github.com/emilk/egui) — table rendering

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
