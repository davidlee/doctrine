# IMP-282: Conformance: exclude slice-own process artifacts from undeclared leads

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at SL-217 audit (RV-270 F-1): `slice conformance` reports the slice's
own `notes.md` and `.doctrine/rfc/011/case-notes.md` (instrumentation mandate)
as undeclared paths. These are expected process artifacts — every audit
re-dispositions the same noise. Classify (or config-exclude) slice-own
`.doctrine/slice/<id>/**` and designated instrumentation paths out of the
undeclared cell, or badge them `process` so they read as informational.

## Pre-implementation findings (2026-07-29)

Established while triaging this against IMP-268; none of it is derivable from the
card above, and one item is a live hazard.

### The hazard — do NOT exclude in the shared predicate

`src/conformance.rs` exposes **two** entry points over ONE match implementation
(`compile` / `matched_selector`), and they have very different blast radii:

| entry point | consumers | kind |
|---|---|---|
| `compute` | `slice.rs:2884` (registry path, via `conformance_outcome`), `slice.rs:2968` (`--against <range>`) | display |
| `undeclared_paths` | **`src/mcp_server/worker_commit.rs:115`**, **`src/worktree/import.rs:107` + `:159`**, `src/plan.rs:320` | **gate** |

Two of `undeclared_paths`' consumers are the gates that stop a worker landing
paths its slice never declared — including authored `.doctrine/**`. Excluding
slice-own `.doctrine/slice/<id>/**` in `compile`/`matched_selector` would punch a
hole straight through `worker_commit` and the import belt. **The exclusion must
attach to the display side only.** This is the whole design call, and it is the
difference between a cosmetic cleanup and a quiet integrity regression.

Secondary decision on the same axis: `slice conformance --against --strict` bails
on a non-empty undeclared cell (`slice.rs:2970`). It reads `compute`, so a
display-side exclusion loosens `--strict` too. Probably wanted (it is an audit
convenience, not a worker gate) — but decide it deliberately rather than
inheriting it.

### The card's second example path is stale

`.doctrine/rfc/011/case-notes.md` is no longer where instrumentation goes. Capture
cut over to **`.doctrine/observations/records/**`** (per CLAUDE.md; records are
*authored* — committed and diffable — so they land inside phase boundaries and
generate the identical noise under a new path). The historical corpus stays at
`case-notes.md` but receives no new entries.

⇒ a hardcoded pair of paths would have been wrong on arrival. Wants a config key
or a classification rule, and STD-001 wants it single-sourced and named, not
inline globs at the call site.

### Read the existing corpus first — this is a known family

Four memories already document undeclared-cell noise; do not rediscover them:

- `mem_019f239c569b75239987428d47b11f8f` (fact, high) — REV-only / governance-output
  slice's own deliverable necessarily reads undeclared. **Carries the triage order
  for undeclared causes, and prescribes the current workaround: dispose `aligned`,
  not `tolerated`.**
- `mem_019f031a315c7803900fcf398092e674` — noise from boundary start-oid pollution
  (foreign commits inside `start..end` when `edge` advances mid-phase).
- `mem_019f0d369fe97231a788b89d56629d43` — shared-branch sweep-in of concurrent
  foreign-slice commits.
- `mem_019f27b5d15b7143a32dbb276d0bd28e` (pattern, high) — the funnel import scope
  belt's counterpart guidance: omit `--slice` for legitimately-coupled
  non-selector paths, dispose undeclared `aligned` at audit.

**Framing that follows:** the workaround is already documented and works. This item
is therefore **ergonomics — ending a repeated per-audit disposition — not a
correctness fix.** Size the change to that. It argues for *badging* `process` over
config-excluding: badging keeps the path visible and drops it out of the
noise-that-needs-disposing, while an exclusion hides information the auditor may
still want.
