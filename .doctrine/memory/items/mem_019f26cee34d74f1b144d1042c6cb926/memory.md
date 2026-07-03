# prompt explain is pre-suppression diagnostic; only prompt resolve applies the replaces graph

`prompt explain` and `prompt resolve` see the corpus at **different stages** of the
hymn cascade. Getting them confused sends verification at the wrong verb (SL-193's
own design.md did exactly this — 3 loci corrected in RV-239).

- **`prompt explain`** — prints the **raw ranked active set**. For an exposed slot
  with a framework twin + a user twin carrying `replaces = <own slot>`, explain
  shows **both**:
  ```
  role/worker   prov=Framework spec=([:1],0) rank=1
  role/worker   prov=User      spec=([:1],0) rank=2 ★ WINNER
  ```
  The `★ WINNER` / `rank` is the **provenance tiebreak in the precedence key — an
  ordering term, not suppression** (INV-2: only `replaces` suppresses). Explain does
  **not** apply the `replaces` graph. Framework twin present, ranked-but-not-dropped.

- **`prompt resolve`** — applies the `replaces` suppression graph. The self-`replaces`
  user twin suppresses its framework origin ⇒ the slot emits **once** (user body).
  This is where single-emit / override actually happens.

## Verify hymn suppression / override at these verbs

- `prompt resolve --role <r> [--harness/--model ...]` → assert single emit / user body wins.
- `prompt check` (⇒ `validate_replaces`, `src/hymns.rs`) → assert legality (every
  `replaces` is the unique-most-specific active snippet of its slot, INV-3; guards
  against `NonTopReplacer`/cycle that would make `resolve` error).
- **Never** assert suppression via `prompt explain` — it is a pre-suppression diagnostic.

Related: seal drops the user twin pre-match (framework wins); expose keeps it with
self-`replaces` (user wins) — the single-emit mirror pair (REV-019 / SL-193).
