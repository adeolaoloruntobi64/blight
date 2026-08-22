# Blight

Blight is an experimental Rust and JavaScript workspace for multi-protocol proxy services, transport tooling, and a small adblock/WASM integration layer.

## Background

This project began as a personal proxy and content access experiment. It was originally built to bypass restrictive school network filters, with a focus on making Wikipedia and other web content accessible through Bare, Wisp, and Wsproxy transports.

Over time the codebase grew, adding adblocking, transport support, and frontend-serving capabilities. The project was later revisited and refreshed as a smaller, more maintainable collection of server and transport components.

## Overview

The repository contains a Rust workspace and a separate JavaScript workspace.

- `server/`: a proxy server binary named `blight` that exposes multiple proxy endpoints and serves optional static frontend assets.
- `crates/`: reusable Rust crates for proxy transports, shared utilities, caching, and client support.
- `packages/`: a JavaScript workspace with transport packages such as a Bare transport implementation and related client tooling.

## What it includes

- A Rust proxy server with support for:
  - Bare protocol (TOMP) versions v1, v2, and v3
  - Wisp protocol versions v1 and v2
  - Wsproxy protocol v1
- A static client service for serving web frontend assets from a local directory.
- Shared utilities in `crates/common` for DNS resolving and IP handling.
- A Wisp multiplexing library in `crates/wisp-mux`.
- An adblock engine integration under `crates/vanguard` using Rust, WASM, and JavaScript.

## Workspace layout

### Rust workspace

- `server/`: main Axum-based proxy server binary
- `crates/common/`: shared utilities, DNS resolver, and IP helpers
- `crates/static-client-tower-axum/`: static file server adapter for Axum
- `crates/tomp-http-tower-axum/`: Bare/TOMP HTTP and WebSocket server implementation
- `crates/wisp-mux/`: reusable Wisp multiplexing library
- `crates/wisp-tower-axum/`: Wisp server adapter for Axum
- `crates/wsproxy-tower-axum/`: WSProxy server adapter for Axum
- `crates/vanguard/`: adblock engine bindings for Rust/WASM/JS
- `crates/tower-etag-cache-0.1.0/`: cache provider utilities for ETags

### JavaScript workspace

- `packages/`: workspace root
- `packages/vanguard/`: Vanguard compiled to wasm for use in js
- `client/`: The frontend (WIP)

## Usage

### Build the Rust workspace

From the repo root:

```bash
cargo build
```

### Run the server

From the repo root:

```bash
cargo run -p blight -- --socket 0.0.0.0:3000
```

### Common runtime options

```bash
cargo run -p blight -- --socket 0.0.0.0:3000 \
  --frontend-files-dir ./client/ \
  --bare-prefix /bare/ \
  --wisp-prefix /wisp/ \
  --wsproxy-prefix /wsproxy/ \
  --allow-udp \
  --allow-non-internet-ports \
  --allow-non-global-ip \
  --auth-path ./auth.txt
```

#### Server CLI flags

- `--socket`, `-s`: socket binding address (default: `127.0.0.1:3000`)
- `--frontend-files-dir`, `-f`: static frontend directory (default: `./client/`)
- `--bare-prefix`, `-b`: Bare endpoint prefix (default: `/bare/`)
- `--wisp-prefix`, `-w`: Wisp endpoint prefix (default: `/wisp/`)
- `--wsproxy-prefix`, `-x`: Wsproxy endpoint prefix (default: `/wsproxy/`)
- `--extra-bare-meta`, `-e`: add extra metadata to Bare WebSocket responses
- `--allow-non-global-ip`, `-g`: allow non-globally-routable IP addresses
- `--allow-udp`, `-u`: enable UDP support for Wisp/Wsproxy
- `--allow-non-internet-ports`, `--allow-non-internet-ports` / `-p`: allow ports other than 80/443
- `--ws-max-message-size`, `-m`: websocket max message size in bytes (default: `1048576`)
- `--auth-path`, `-a`: path to a username/password file for Wisp auth

## Notes

- This repository is experimental and not intended as a polished production deployment.
- The Rust server is designed to expose multiple proxy transports from a single process.
- The default frontend path is `./client/`, but the repository does not include a complete production frontend by default (WIP).

## Status

- `Bare` / `TOMP`: v1, v2, v3 supported
- `Wisp`: v1, v2 supported
- `Wsproxy`: v1 supported
- `Vanguard` engine: present as Rust/WASM/JS integration
- `Static client`: supported through `static-client-tower-axum`

## References

- https://github.com/MercuryWorkshop
- https://github.com/titaniumnetwork-dev
- https://github.com/tomphttp/specifications
- https://github.com/MercuryWorkshop/scramjet
- https://docs.crllect.dev/