# IMP-315: Projected reference docs under `.doctrine/` are stale and unrefreshable post-SL-227

Surfaced by the SL-229 audit (RV-306 F-7). Not an SL-229 defect — SL-229 merely
exposed it.

## Problem

Nine reference docs sit tracked under `.doctrine/` as projected copies of the
`install/` masters. Comparing each against its master today:

| doc | state |
|---|---|
| `glossary.md` | **stale** |
| `review-ledger.md` | **stale** |
| `using-doctrine.md` | **stale** |
| `routing-process.md` | **stale** |
| `dispatch-mechanics.md` | **stale** |
| `harvest.md` / `boot-footer.md` / `model-band.md` | current |
| `governance.md` | differs by design — user-owned, seeded once, never overwritten |

After SL-227 (minimal projection, ADR-019), `install --dry-run` emits only
`doctrine.toml` and `project-orientation.md`. Reference docs are no longer
projected, so the five stale copies **can never refresh**.

The drift is not marginal. `.doctrine/glossary.md` is missing SL-229's
`research/` layout line *and* the entire knowledge-record kind table
(`ASM`/`DEC`/`QUE`/`CON`/`EVD`/`HYP`) and the knowledge lifecycle vocabulary.

The hazard is that agents still read them. The boot snapshot says "**Reference
docs (read on demand).** `glossary.md` — kinds, ids, full reference forms,
verification taxonomy", which resolves to the stale on-disk file; the SL-229
auditor read `.doctrine/review-ledger.md` on exactly that instruction. The
copies are correct *enough* not to announce themselves as stale, which is the
worst failure mode.

The published assets are fine: `doctrine library show reference/glossary.md`
serves the current master from the embedded corpus.

## Options

1. **Delete the stale projected copies** and repoint boot's reference-doc line
   at `doctrine library show reference/<name>` — consistent with ADR-019's
   minimal-projection intent; costs a `library show` per read instead of a
   `Read`.
2. **Restore projection for the reference tier only** — cheapest for agents,
   but re-opens the projection surface ADR-019 deliberately narrowed.
3. **Stale-detection**: a `doctrine doctor` check comparing on-disk copies to
   published bytes, warning where they diverge — cheap, and leaves the
   projection decision open.

Sits next to IMP-312 (SL-227 library/install hardening) and IMP-313.
