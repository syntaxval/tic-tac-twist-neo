# ADR 004: CSS Grid and SVG in Svelte 5 instead of WebGL

## Context

A 3x3 to 10x10 strategy board needs sharp rendering across mobile pixel densities, responsive sizing, keyboard accessibility, and smooth telemetry updates.

## Decision

Render the board with semantic HTML (`role="grid"` and `role="gridcell"`), CSS Grid, and an SVG winning-line overlay inside Svelte 5 rather than WebGL.

## Consequences

- Native touch targets, ARIA labels, roving-tabindex arrow-key navigation, and light/dark `color-scheme` support work out of the box.
- Svelte 5 compiles reactive state into direct DOM updates with no virtual DOM diffing, keeping the JS bundle at 24 kB gzipped and avoiding GPU battery drain on mobile devices.
- If a 3D board or shader renderer is added later, it can connect to the same `EngineClient` Web Worker without changing engine code.
