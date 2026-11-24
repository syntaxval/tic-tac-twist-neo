# Tic-Tac-Twist Neo

A three-player strategy game backed by a Rust `u128` bitboard engine. The same engine compiles to WebAssembly for the browser and to a native binary for the terminal.

[![Play live](https://img.shields.io/badge/Play%20Live-GitHub%20Pages-7c3aed?style=for-the-badge)](https://syntaxval.github.io/tic-tac-twist-neo/)
[![Rust](https://img.shields.io/badge/Rust-Engine%20%2B%20CLI-000000?style=for-the-badge&logo=rust)](crates/engine)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-57%20kB%20gzip-654ff0?style=for-the-badge&logo=webassembly)](crates/wasm)
[![Svelte 5](https://img.shields.io/badge/Svelte%205-24%20kB%20gzip-ff3e00?style=for-the-badge&logo=svelte)](web)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue?style=for-the-badge)](LICENSE)

[Play the web version on GitHub Pages](https://syntaxval.github.io/tic-tac-twist-neo/). The engine runs locally inside a browser Web Worker with no backend server.

## Overview

Tic-Tac-Twist extends Tic-Tac-Toe to 3 players, board sizes from 3x3 to 10x10, configurable K-in-a-row win lengths, and an optional Four Corners win rule.

A single pure-Rust crate (`crates/engine`) owns all board state, win detection, and AI search. Two thin adapters sit on top of it:

- `crates/wasm` plus `web/` runs the engine in a Web Worker behind a 24 kB gzipped Svelte 5 interface.
- `crates/cli` compiles the engine into a native terminal binary with ANSI colors and Unicode box drawing.

## Technical details

| Area | Implementation |
| :--- | :--- |
| Board representation | One `u128` bitboard per player holds up to 100 cells (10x10). Win checks use a single bitwise mask test, coordinate conversion runs in O(1) integer math, and search make/unmake allocates no heap memory. |
| Three-player AI | A tactical pass checks for an immediate winning move or a one-move opponent threat. Deeper search supports Paranoid Alpha-Beta (iterative deepening, time budget, Zobrist hashing, 65,536-entry transposition table), MaxN (per-player score vectors), and a 1-ply Greedy heuristic. |
| Search telemetry | Every AI turn reports algorithm name, depth reached, nodes evaluated, transposition table hits, elapsed milliseconds, and nodes per second. |
| Win rules | `LinesRule` and `FourCornersRule` implement a `WinRule` trait and precompute bitmasks from the active board size. |
| Web UI | Scoped CSS Grid and SVG in Svelte 5 with zero UI library dependencies, automatic light and dark `color-scheme`, roving-tabindex arrow-key navigation, and `appearance: base-select` dropdowns with a fallback. |

## Architecture

```mermaid
flowchart LR
    subgraph Core["crates/engine (Pure Rust, no I/O)"]
        Board["u128 Bitboards"]
        Rules["WinRule Strategies"]
        Game["Game State Machine"]
        AI["Paranoid, MaxN, Greedy + Telemetry"]
    end

    subgraph Native["crates/cli"]
        CLI["ANSI + Unicode Terminal UI"]
    end

    subgraph Browser["crates/wasm + web/"]
        WASM["wasm-bindgen (57 kB gzip)"]
        Worker["Web Worker"]
        UI["Svelte 5 + CSS Grid/SVG (24 kB gzip)"]
    end

    Core --> CLI
    Core --> WASM --> Worker <--> UI
```

```text
tic-tac-twist-neo/
├── crates/
│   ├── engine/          # Pure Rust core: bitboards, rules, state machine, AI
│   ├── cli/             # Native terminal binary
│   └── wasm/            # wasm-bindgen bridge
├── web/                 # Svelte 5, TypeScript, and Vite static web app
├── docs/adr/            # Architecture decision records
└── .github/workflows/   # Automated tests, WASM build, and GitHub Pages deploy
```

## Quickstart

### Play online

Open [syntaxval.github.io/tic-tac-twist-neo](https://syntaxval.github.io/tic-tac-twist-neo/) in a desktop or mobile browser.

### Play in the terminal

```sh
cargo run --release -p tic-tac-twist-cli -- --size 5 --win-length 4 --ai paranoid
```

Run with `--help` to list all flags (`--size`, `--win-length`, `--four-corners`, `--ai paranoid|maxn|greedy`, `--time-ms`).

### Run the web app locally

Requires Rust with the `wasm32-unknown-unknown` target, `wasm-pack`, Node.js 22+, and `pnpm`:

```sh
wasm-pack build crates/wasm --target web --out-dir ../../web/src/wasm
pnpm --dir web install
pnpm --dir web dev
```

### Run tests and lints

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir web check
pnpm --dir web build
```

## Architecture decision records

- [ADR 001: `u128` bitboards for 3x3 to 10x10 boards](docs/adr/001-u128-bitboards.md)
- [ADR 002: Three-player AI with Paranoid Alpha-Beta and MaxN](docs/adr/002-three-player-ai-paranoid-vs-maxn.md)
- [ADR 003: One Rust engine for WASM Web Worker and native CLI](docs/adr/003-rust-wasm-web-worker.md)
- [ADR 004: CSS Grid and SVG in Svelte 5 instead of WebGL](docs/adr/004-dom-svg-over-webgl.md)

## License

Apache License, Version 2.0. See [LICENSE](LICENSE).
