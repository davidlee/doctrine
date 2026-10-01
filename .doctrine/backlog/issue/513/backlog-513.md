# ISS-513: Section body trailing newline doubles blank lines on materialise

## What

A section body submitted with a trailing newline renders with doubled blank lines when
the run is materialised to `design.md`.

## Fix sketch

Normalise trailing whitespace on section bodies at write or render.

## Evidence

`01a0d8c7` (2026-09-25).
