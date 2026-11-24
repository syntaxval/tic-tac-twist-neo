# Tic-Tac-Twist Neo

A Principal-level re-engineering of the original 2019 take-home interview project (`tic-tac-twist`), preserving the original snapshot in git history under tag [`v0.1.0-interview-2019`](../../tags).

## Architecture

```text
tic-tac-twist-neo/
├── crates/
│   ├── engine/   # Pure Rust: u128 bitboards, composable WinRules, Paranoid/MaxN/Greedy AI + telemetry
│   ├── cli/      # Native terminal UI (ANSI colors + Unicode grid, parity with 2019 CLI)
│   └── wasm/     # Thin wasm-bindgen bridge compiled to WebAssembly
├── web/          # Svelte 5 + TypeScript + Vite UI (CSS Grid + SVG, Web Worker, live AI telemetry)
└── docs/adr/     # Architecture Decision Records defending every key trade-off
```

### Key Upgrades over `v0.1.0-interview-2019`
- **`u128` Bitboard Engine:** 3×3 through 10×10 (up to 100 cells) fits in a single `u128` per player with $O(1)$ coordinate conversion and zero-allocation make/unmake.
- **Real 3-Player AI:** Replaces the broken 2019 minimax with a tactical win/block fast path + selectable **Paranoid Alpha-Beta** (iterative deepening + time budget + Zobrist Transposition Table), **MaxN**, and **Greedy** strategies, all reporting live telemetry.
- **Composable Rules:** `LinesRule` (with configurable `winLength`) and `FourCornersRule` use actual board dimensions — zero global config coupling.
- **Strict Domain / I/O Separation:** One pure Rust core drives both the native CLI and the browser Web Worker WASM app.

## Quickstart

### 1. Run the Rust test suite
```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### 2. Play in the Terminal (Native CLI)
```sh
cargo run --release -p tic-tac-twist-cli -- --size 5 --win-length 4 --ai paranoid
```
Run with `--help` for all options (`--size`, `--win-length`, `--four-corners`, `--ai`, `--time-ms`).

### 3. Play in the Browser (Web / Mobile UI)
```sh
wasm-pack build crates/wasm --target web --out-dir ../../web/src/wasm
pnpm --dir web install
pnpm --dir web dev
```

## Architecture Decision Records
- [ADR 001: `u128` Bitboards](docs/adr/001-u128-bitboards.md)
- [ADR 002: 3-Player AI — Paranoid Alpha-Beta vs. MaxN](docs/adr/002-three-player-ai-paranoid-vs-maxn.md)
- [ADR 003: Rust Engine → WASM Web Worker + Native CLI](docs/adr/003-rust-wasm-web-worker.md)
- [ADR 004: CSS Grid + SVG (Svelte 5) Over WebGL](docs/adr/004-dom-svg-over-webgl.md)

## License
Apache License, Version 2.0. See [LICENSE](./LICENSE).
