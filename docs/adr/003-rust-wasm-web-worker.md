# ADR 003: Single Rust Engine → WASM Web Worker + Native CLI (No Backend for v1)

## Context
We want near-native AI search throughput, a responsive web/mobile UI, and parity with the 2019 terminal CLI — without duplicating game logic or operating a server for local play.

## Decision
Build `crates/engine` as a pure, I/O-free Rust crate and compile it to two thin targets:
- `crates/cli`: Native terminal binary with ANSI colors and Unicode box drawing.
- `crates/wasm`: `wasm-bindgen` module executed inside a browser **Web Worker**, communicating with the UI via typed async messages.

## Consequences
- **Zero network latency & offline-ready:** All AI search runs locally on the client device with zero main-thread jank.
- **Single source of truth:** Rules, bitboards, and AI behavior are identical in CLI and Web.
- **Future multiplayer path:** Because `crates/engine` has zero I/O, it can later be embedded in an Axum/WebSocket server without rewriting domain logic.
