# ADR 003: One Rust engine for WASM Web Worker and native CLI

## Context

The project needs fast AI search, a responsive browser and mobile interface, and terminal CLI support without duplicating game rules or running a backend server for local play.

## Decision

Keep `crates/engine` free of I/O and compile it to two targets:

- `crates/cli` builds a native terminal binary with ANSI colors and Unicode box drawing.
- `crates/wasm` exposes a `wasm-bindgen` wrapper that runs inside a browser Web Worker and exchanges plain JSON-compatible messages with the UI.

## Consequences

- AI moves run locally on the client with zero network latency and do not block the browser main thread.
- The CLI and web app share identical board rules, bitboards, and AI search code.
- Because `crates/engine` performs no I/O, a future WebSocket server can import the same crate without rewriting game logic.
