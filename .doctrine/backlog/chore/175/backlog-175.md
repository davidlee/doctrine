# CHR-175: Add python3 to the jail

## What

The bubblewrap development jail (`flake.nix`) has no `python3`. Agents reach for it for
structured, line-precise file surgery, fail, and fall back to perl/sed/awk — where they
hit delimiter traps (`s|||`, `#` vs Rust attributes) and `perl -0pi` half-applies a
mutation silently.

## Fix sketch

Add `python3` (no extra packages needed) to the jail's package set in `flake.nix`. If
excluding it is deliberate, say so in AGENTS.md's jail section so agents stop trying.

## Evidence

12 observations, 2026-08-15 → 2026-09-28, mostly capsule workers: `01a00610`,
`01a0ade9` (perl half-applied a mutation), `01a0ba84`, `01a0de02`, `01a0e34c`, `01a0ea38`.
