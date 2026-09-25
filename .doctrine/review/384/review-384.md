# Review RV-384 — design of SL-265

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

## Response integration (2026-09-25)

Both findings were dispositioned `route:control fixed`. For the record, the fix
landed **in the design itself** as well as in the criterion they name, before the
lock:

- `F-1` — `sec-3`, `sec-7` and `DEC-298` now iterate the `KINDS` table (the set
  `parse_resolvable_ref`/`kind_by_prefix` resolve against), with a negative
  control on a synthetic unrouted prefix.
- `F-2` — `sec-5` and `sec-7` now sequence `coverage record` **then**
  `coverage verify 265`, exiting on the cell reading `Verified`.

The integration moved three section fingerprints, so this pass no longer covers
the current document: the run lamps it `STALE`. The amended sections are covered
by the user's section attestations at lock; the raised criterion for each finding
still needs transcribing onto a phase criterion at `/plan`, which is when the
raiser verifies it.

## Second pass — independent adversarial read (2026-09-25)

An independent review agent (read-only, `./scripts/pi-research`) was run over
`design.md`, the scope and every cited governance entity, with a hostile brief.
It returned 12 falsified claims plus 3 nits; each was verified against source
before integration and ledgered here as `F-4`–`F-15`.

The four load-bearing repairs:

- **`F-4` (blocker)** — the delegation table routed 23 of 24 `KINDS` rows: `RFC`
  was missing while the design's own worked example is `doctrine show RFC-031`.
- **`F-5`** — the totality assertions were placed in `tests/`, where `pub(crate)`
  `route()`/`KINDS` are unreachable; they are a unit test.
- **`F-6`/`F-7`** — the governance sequence was not executable (one `change add`
  row per invoke; `revision new` omitted) and mis-stated the new requirement's
  landing status (`pending`/`Coherent`, not `Indeterminate`).
- **`F-8`** — the chosen home *would* add a `commands → governance` edge; routing
  through the per-kind wrappers keeps the design's own no-new-edge claim true,
  and the top-level alternative's real cost is an `Unclassified` tier row.

`F-1`, `F-2`, `F-5` and `F-13` are instrument-routed (`route:control`) and stay
`answered` until `/plan` transcribes their criterion onto a phase; the rest are
prose-repaired and `verified`. The pass is `STALE` against the section set it was
opened over — the human attestations at lock cover the integrated document.
