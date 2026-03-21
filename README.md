<h1 align="center">
  <a href="https://rewire.run/">
    <img alt="banner" src="https://github.com/user-attachments/assets/4859413d-89b2-424c-a378-8a15260de384">
  </a>
</h1>

<p align="center">
  <a href="https://github.com/rewire-run/rewire-viewer/actions/workflows/ci.yaml">
    <img alt="CI" src="https://github.com/rewire-run/rewire-viewer/actions/workflows/ci.yaml/badge.svg">
  </a>
  <a href="https://github.com/rewire-run/rewire-viewer/releases/latest">
    <img alt="Version" src="https://img.shields.io/badge/version-v0.2.0-green">
  </a>
  <a href="https://github.com/rewire-run/rewire-viewer/blob/main/LICENSE">
    <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue">
  </a>
  <a href="https://pixi.sh">
    <img alt="Powered by" src="https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json">
  </a>
</p>

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

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
