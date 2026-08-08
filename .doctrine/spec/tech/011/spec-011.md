# SPEC-011: Boot snapshot

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The boot snapshot is the container that projects stable governance state into a
cache-friendly session-start prefix, so an agent pays for governance once per
change rather than once per session. It assembles `.doctrine/state/boot.md` — a
**derived, runtime-state** artefact (gitignored, `rm -rf`-able, never
authoritative) — that each harness `@`-imports ahead of its committed
instructions. It sits beneath the whole-system root (SPEC-003) and rides the
governance kinds (ADR, policy, standard, memory) and the install/listing
substrate for their row data; this spec restates none of that and owns only what
is specific to *the projection and its wiring*: the pure assembly seam, the
content-diff cache key, the section source-kind taxonomy and its marker
fallback, the `@`-import + Claude hook-set installer, and the `--check` disk
sentry.

## Responsibilities

Mirrors the structured `responsibilities` list: assemble the snapshot as a pure
deterministic projection; order the build-volatile exec-path section last;
project each governance kind through one status-filtered arm; fall back to a
fixed marker on any miss; write only on content change; wire the `@`-import and
the Claude hook set through `boot install`, into the scope-selected settings file
and sweeping the abandoned sibling; and run `boot --check` as a disk-scoped
sentry.

### The pure assembly seam

The snapshot is a pure projection split from its impurity by the house rule. A
pure assembler — `boot_sequence` + `render_boot` — takes already-produced
section bodies and yields a deterministic string (a header comment, the
`# Doctrine Boot Context` title, then each section as `## {heading}\n{body}`)
with no clock, rng, or disk. The thin impure shell (`produce`,
`write_if_changed`, `run`) gathers each body, resolves `current_exe()`, and
reaches disk through the shared `fsutil::write_atomic` seam. `boot_sequence`
declares the ordered `(heading, SourceKind)` list; a new governance kind appends
one `GovRows` row and nothing else changes. The `Invoking doctrine`
(`ExecPath`) section is deliberately **last**: the resolved exec path is
build-volatile, so ordering it at the tail confines a path change to the
snapshot's end and leaves the whole governance prefix above it cache-warm.

### The section source-kind taxonomy and marker fallback

A `SourceKind` names where each section's body comes from: `Static(name)` reads
a canonical embedded digest by `install/`-relative filename (the embed, never
disk, so it is not user-clobberable); `Governance` reads the user-owned
`.doctrine/governance.md` pointer layer from disk; `GovRows(kind, set)` projects
a numbered governance kind filtered to a status set; `Memories` projects active
memory pointers (a distinct `None`-scope signature); and `ExecPath` carries the
resolved binary path. Every producer is total: a miss never panics. The
`section_or_marker` helper maps an error OR an empty listing alike to a fixed
`marker(heading)` — `<!-- {heading}: not yet populated -->` — so a missing
source degrades to a benign, *byte-stable* placeholder rather than a crash or a
variable body that would bust the cache.

### Governance projection through one status-filtered arm

Each numbered governance kind projects through a single `GovRows` arm that
builds a `listing::ListArgs` directly — boot is a declared non-clap consumer of
the shared listing model. The arm carries the kind descriptor plus an explicit
in-force status SET: accepted ADRs, required policies, default+required
standards. The explicit set is what reveals these rows *past* each kind's
default list hide-set, which is exactly the boot intent. Memory is its own arm,
filtered to `active` only — an explicit boot predicate decoupled from the CLI
`memory list` default (which keeps `draft` visible): boot is an agent-context
*producer* and unreviewed `draft` memory must not leak into the snapshot. The
`.doctrine/governance.md` body is the editable user-owned layer projected as the
`Governance` section — distinct from the embedded `routing-process.md` digest.

### The content-diff cache key

The snapshot writes to `.doctrine/state/boot.md` through `write_if_changed`,
which writes only when the recomputed content differs from what is already on
disk. The content diff *is* the cache key: a no-op rewrite would needlessly bust
the agent's cached session-start prefix, so an unchanged regenerate reports
`Unchanged` and touches nothing. `doctrine boot` resolves the root and
`current_exe()`, regenerates, and reports `Wrote` or `Unchanged`.

### `boot install` — import wiring and hook merge

`boot install` resolves target harnesses (explicit `--agent` wins, else
auto-detect by `.claude/` / `.codex/` markers), then does two things behind a
pure-plan / imperative-apply split. First it prepends the `@.doctrine/state/boot.md`
import **once** into each harness's committed file — Claude reads `CLAUDE.md`,
codex `AGENTS.md`, one file per harness so the snapshot never inlines twice —
canonicalising each target so a `CLAUDE.md → AGENTS.md` symlink is updated
through to its single inode and same-inode targets dedup to one write. The
prepend is idempotent: a file already carrying the ref line plans no write.
Second, for Claude it merges doctrine's owned **hook set** — every spec's entries,
in matcher order, across every event it declares — into the settings file the
`[install] claude-settings-scope` key selects. Entry identity is the set, so a
hand-edit that deletes one entry of a multi-matcher spec is healed on the next
install. Each spec is recognised and refreshed via an ownership predicate over its
`command`, preserving every foreign hook and unrelated key by mutating the JSON at
the narrow path, and failing soft — a malformed settings file is left untouched and
a manual-paste snippet covering the whole matcher set is printed instead.

The scope key is the only selector (no flag): absent key, absent `[install]` table
and absent `doctrine.toml` all yield project `.claude/settings.json`, and the
alternative is `.claude/settings.local.json`. The file's tracked-ness fixes the
command form on `baked ⟺ gitignored` — the committed project file gets the portable
`${DOCTRINE_BIN:-doctrine}` literal, the gitignored local file a baked exec path —
and `worktree.baseRef` follows the same scope, so doctrine writes exactly one of
the two files.

Writing one scope then **evicts** this project's owned entries from the other, by
the same ownership predicate and gated on the target write having landed, so a
failed write never strands the operator with no working copy. The installer
announces its target file and the key that changes it before any hook line, and
riders report what the sweep removed, could not read, or did not attempt. A
`worktree.baseRef` stranded in the abandoned file is reported rather than swept,
with the message stating which of the two values actually governs.

Codex is import-only (no hook set — one hook, baked form). A single harness's
refresh failure is isolated and printed; the others still run.

### `boot --check` — the disk sentry

`boot --check` recomputes the snapshot in memory and compares it to the
on-disk bytes: `stale` when they differ (an unreadable file counts as stale, the
recompute differing from nothing), plus the set of sections whose body is still
a marker (unpopulated). It reports DISK-scoped wording only — it never claims
the *current session's* already-inlined prefix is fresh, because an in-session
edit lags until `/clear` or restart. Closing that lag is the freshen-now ritual
(`/canon`: regenerate, then `/clear`), not this verb's job.

## Concerns

- **In-session lag.** The on-disk snapshot and the agent's inlined prefix
  diverge for up to two sessions after a governance edit; `--check` only sees
  disk, and the live-prefix freshen is a separate `/clear`/restart ritual, not
  something this container can assert.
- **Cache-key fragility.** Any non-deterministic body (a clock, an unsorted
  listing, a trailing blank line) would bust the content-diff cache every
  session; the marker fallback and trailing-newline trim exist to keep
  `render_boot` byte-stable.
- **Settings-merge safety.** The hook merge writes into a hand-editable JSON
  file it does not own; a malformed or oddly-typed `hooks`/`<event>` structure
  must fail soft (print-and-skip), never clobber foreign content.
- **The sweep is the one destructive write.** Evicting the abandoned scope is
  what keeps a scope flip from leaving two live copies of every hook — a failure
  that presents as mild slowness and nothing else. It is bounded by the same
  ownership predicate as the merge and gated on the target write landing, so its
  blast radius is doctrine's own entries and never the only working copy.

## Hypotheses

- **A pure projection with a content-diff write is the cache lever.** Keeping the
  assembler pure and writing only on change is preferred over an
  always-rewrite, because the agent's prefix is cached on content identity — an
  unchanged regenerate must produce identical bytes to keep the cache warm.
- **Ordering the volatile section last localises churn.** The build-volatile
  exec path is the one body that changes without a governance change; placing it
  at the tail is preferred so its diff never invalidates the governance prefix
  above it.
- **The snapshot is derived, never authoritative.** The projection lives in the
  gitignored runtime-state tree and is regenerable from governance state, so it
  is preferred to treat it as a disposable cache rather than an authored
  artefact — reconciling it is regeneration, not editing.

## Decisions

- **D1 — the snapshot is a pure projection of governance state.** Assembly takes
  no clock/rng/disk; the impure shell gathers bodies and resolves the exec path.
  This keeps the output deterministic so the content-diff cache key holds.
- **D2 — write only on content change.** `write_if_changed` is the cache
  contract: a no-op rewrite would bust the cached prefix, so an unchanged
  regenerate touches nothing and reports `Unchanged`.
- **D3 — the build-volatile exec path is ordered last.** It is the one
  non-governance body; tailing it confines a path change to the snapshot end and
  leaves the governance prefix cache-warm.
- **D4 — a missing source is a marker, not a crash.** Every producer is total;
  `section_or_marker` collapses error and empty-listing alike to a fixed marker,
  so the snapshot always assembles and stays byte-stable.
- **D5 — `--check` is disk-scoped.** The sentry reports only the on-disk
  snapshot's freshness and unpopulated sections; the live in-session prefix lag
  is closed by the `/clear` freshen ritual, never claimed fresh by this verb.
- **D6 — the installer preserves foreign content and fails soft.** The
  `@`-import is an idempotent dedup'd prepend; the hook merge mutates settings
  JSON at the narrow path, preserving every foreign hook, and prints a manual
  snippet rather than clobbering a malformed file.
