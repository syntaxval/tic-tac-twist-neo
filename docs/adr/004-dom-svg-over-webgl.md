# ADR 004: CSS Grid + SVG (Svelte 5) Over WebGL for the Web/Mobile UI

## Context
A 3×3 to 10×10 strategy board requires crisp rendering across mobile DPRs, responsive scaling, keyboard accessibility, and live AI telemetry updates.

## Decision
Render the board with semantic HTML (`role="grid"` / `role="gridcell"`), CSS Grid, and an SVG winning-line overlay in **Svelte 5**, rather than WebGL.

## Consequences
- **Accessibility & UX:** Native touch targets, screen-reader labels, roving-tabindex arrow-key spatial navigation, and automatic light/dark `color-scheme` support.
- **Performance:** Svelte 5 compiles away the framework (no VDOM diffing) with fine-grained signal updates, keeping bundle size minimal and telemetry updates fast without draining mobile battery.
- **Trade-off:** If a 3D tilting board or GPU shader showcase is desired later, a WebGL renderer can be mounted as an alternate view over the same `EngineClient` Web Worker.
