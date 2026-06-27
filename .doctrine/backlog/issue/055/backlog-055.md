# ISS-055: Config surface split: root doctrine.toml read by dtoml seam, .doctrine/doctrine.toml by priority/config; magic-string path duplication

## Defect

`doctrine.toml` config has **two homes** read by different subsystems:

- **Root `./doctrine.toml`** — read by the `dtoml` seam
  (`src/dtoml.rs`: `DOCTRINE_TOML` const + `read_doctrine_toml_text` →
  `root.join("doctrine.toml")`). Feeds **reservation, dispatch (deliver_to),
  conduct, install, skills, slice** — all via `load_doctrine_toml`.
  Plus two direct stragglers: `src/catalog/hydrate.rs:444`,
  `src/coverage_store.rs:200`.
- **`.doctrine/doctrine.toml`** — read by **priority** (`src/priority/config.rs:113`)
  and the user-facing **`doctrine config` get/set** command
  (`src/commands/config.rs:294,424`).

A user editing `.doctrine/doctrine.toml` (where `doctrine config` writes) sees
`[reservation]` / `[dispatch]` silently ignored — those ride the root reader.
This actually bit us: `reach = "local"` set in `.doctrine/doctrine.toml` did
nothing; the reservation seam kept reading root.

## Decision (User, 2026-06-27)

- **Canonical home: `.doctrine/doctrine.toml`.** Root `./doctrine.toml` is the
  heretic — it should not exist or be read. (Root file already deleted from this
  tree.)
- **Back-compat posture: HARD BREAK.** Read `.doctrine/doctrine.toml` only; an
  existing root `./doctrine.toml` is silently ignored. No fallback, no
  deprecation shim. Existing installs migrate manually.

## Scope of the fix

1. Redirect the **single `dtoml` seam** to `.doctrine/doctrine.toml` (one const /
   one path-helper); the 12 consumers follow for free.
2. Redirect the **2 direct stragglers** (`catalog/hydrate.rs:444`,
   `coverage_store.rs:200`) — and route them through the SAME const, not a new
   literal.
3. **Kill the magic-string duplication** — `"doctrine.toml"` is hand-typed at
   ~20+ sites (incl. all the tests: `dep_seq`, `facet`, `relation`, `slice`,
   `coverage_verify`, `reserve`, `catalog`, `priority/config:227`,
   `commands/config:513`). A single canonical path helper/const must own the
   location so it can never drift into two homes again. **This duplication is the
   root cause** of the split, not an incidental nit.
4. Update the ~20 test write-sites to the new path (behaviour-preservation gate,
   AGENTS.md — suites stay green).
5. Rewrite the docs that now lie: `install/doctrine.toml.example` header and
   `src/dtoml.rs:79` comment both say *"NOT under `.doctrine/`"* — invert them.
   Check ADR-009 / POL-002 references to config location.

## Migration note (immediate consequence)

Root `./doctrine.toml` was deleted **before** the reader moved, so until this
lands the `dtoml` seam reads an absent file → defaults:

- `[dispatch] claude-force-subprocess-dispatch = true` (was in root) is **lost** →
  dispatch reverts to default.
- `[reservation]` reverts to `auto` → review/backlog id-claim verbs error on the
  jail's ssh-disabled origin unless `DOCTRINE_RESERVATION_FALLBACK=1` is set, OR
  the reader is moved to `.doctrine/doctrine.toml` (which already holds
  `reach = "local"`).

Both `[dispatch]` and `[reservation]` keys must land in `.doctrine/doctrine.toml`
as part of (or before) the fix.

## Origin

Surfaced during the SL-164 design inquisition (RV-173) while diagnosing a
reservation-remote error.
