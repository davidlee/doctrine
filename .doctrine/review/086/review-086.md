# Review RV-086 — design of SL-100

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisition arraigns the **design aspect** of SL-100 — the memory lifecycle
verbs (`status`, `edit`, `tag`) and skill updates. The slice claims `design`
status, *locked*, with a prior adversarial pass already folded in (findings F1–F4
in `design.md`; commit `99c8e64c`). A design blessed once is no design above
suspicion; the Inquisition presses where a self-review is most apt to flatter
itself — the load-bearing claims the canon makes about the *existing substrate*,
for a design that misreads the ground it stands on will lead the implementer into
the pit.

**Lines of interrogation:**

1. **The `--key` immutability premise (R1/F1).** The canon swears the scaffold
   "writes `memory_key = ""`" and that `run_edit` "checks `memory_key.is_empty()`
   rather than `Option<>`." Held against the code: `memory_key: Option<String>`
   (`memory.rs:379`); the scaffold *omits* the line when no key is given
   (`render_memory_toml`, `None => String::new()`, `memory.rs:785`); the very
   doc-comment above it (`memory.rs:779`) confesses that `memory_key = ""` *would
   fail `validate_key` on read*. Does the named risk-mitigation survive, or is it
   built on a phantom?

2. **The single-transaction vow (D2).** `edit` swears "a single read→mutate→write
   transaction" (L101) yet folds `--status` by "the same … logic as memory status"
   (L128) — which routes through the **IO** `set_authored_status` (`dep_seq.rs:344`,
   per the `knowledge::run_status` precedent), a *second* independent file write
   and a second `updated` stamp. Pure or not? One write or two?

3. **Record/edit parity on `--key` (D2 table).** The canon cites the private
   `validate_key` (`memory.rs:270`) — which *rejects* a bare key — where record
   normalizes through `normalize_key` (`memory.rs:293`), prepending `mem.`. Will
   `edit --key foo` refuse what `record --key foo` accepts?

4. **Truthfulness of the asserted field paths (INV-schema).** The scaffold template
   carries `[review]` with only `verification_state` — **no `review_by`**. The
   design names `[review].review_by` "replace" and `--review-by ""` to clear. On a
   fresh memory the key is absent: insert, not replace; clear is a no-op. Stated?

Doctrine consulted: ADR-001 (layering), ADR-004 (`superseded_by` carve-out),
ADR-010 (Tier-3 memory labels), the pure/imperative split, the
behaviour-preservation gate, and the real schema in
`.doctrine/templates/memory.toml`.

## Synthesis

**Judgement.** The design of SL-100 is sound in its bones — the verb shapes, the
leaf extraction of `normalize_tag`, the ADR-004 `superseded_by` carve-out, the
behaviour-preservation gate all stand unindicted. But the canon was caught
**bearing false witness about the ground it stands on**. Four deviations were
confessed; none was fatal, all are now reconciled in the canon before the slice
descends to `/plan`.

The gravest, **F-1** (major), was a phantom the *prior* self-review (`design.md`
F1) had already blessed: R1 swore the scaffold "writes `memory_key = """ and that
`run_edit` checks `is_empty()`. The substrate denies it — `memory_key:
Option<String>` (`memory.rs:379`), the line is **omitted** when unset
(`render_memory_toml`, `memory.rs:782`), and an empty string would itself fail
`validate_key` on read (`memory.rs:779`). An implementer obeying the canon would
have built an `is_empty()` guard against an `Option`. **Let this be the lesson
burned into the record: a design reviewed once is not a design absolved — the
self-flattering claim hides exactly where the first pass already looked and
nodded.** The canon now decides immutability on the `Option` (`is_some()`), as the
code demands.

The remaining three were lesser taints, all corrected: **F-2** (minor) — `edit`'s
"single transaction" vow reconciled with `--status` by naming the **pure**
`dep_seq::apply_status` as the composition point (not the IO `set_authored_status`,
which would write twice); **F-3** (minor) — `edit --key` now cites `normalize_key`
for record/edit parity, not the private `validate_key` that rejects a bare key;
**F-4** (nit) — the `--review-by` row now reads "insert-or-replace; clear is a
no-op", honouring a scaffold template that omits `review_by` at record time.

**Corrective sequence (penance, applied).**
1. ✅ R1, F1, D2 `--key` invariance rewritten to the `Option` model; empty-string
   fiction struck. (`design.md`)
2. ✅ D2 `--status` delegation rerouted to the pure `apply_status`;
   `memory_status_transition` declared the shared pure core.
3. ✅ D2 `--key` validation re-cited to `normalize_key`.
4. ✅ D2 `--review-by` row reworded to insert-or-replace.

**Verification at /plan.** The plan must carry tests that pin the corrected canon:
`edit --key` refused on a `Some` key / accepted (and `mem.`-prefixed) on a `None`
key; `edit --status` produces exactly **one** file write and **one** `updated`
stamp; `edit --review-by` on a freshly-recorded memory inserts the key. These are
already half-present in the Verification-alignment table (L348-349) — they now bind
to truthful behaviour.

**Standing risks, consciously tolerated.**
- **Two writes on `supersede` (D1).** `memory status … superseded --by` performs
  two independent IO writes — `append_memory_relation` then `set_authored_status`
  — and is therefore non-atomic, unlike `edit`'s single transaction. The design's
  ordering rationale (relation first) makes the failure mode benign (a
  `superseded_by` edge on a still-`active` record, re-runnable idempotently). Both
  reuse existing seams; generalising them into one transaction is not worth the
  coupling. **Tolerated.**
- **No status-transition legality matrix.** The design permits any of the 6 states
  from any state (e.g. `superseded` → `active`). This follows the
  `knowledge::run_status` precedent (vocab gate only, no matrix); `is_hidden`
  (`memory.rs:109`) is a list-visibility set, not a terminality guard. Consistent
  with precedent — **tolerated**, noted for a future hardening slice should
  resurrection of dead memories prove a hazard.

**Harvest.** Nothing durable beyond this slice — the corrections live in `design.md`
where they belong, and the standing risks are recorded here against the subject.
No memory, no backlog item warranted. A near-clean trial: the heresy was textual,
the substrate honest.

> **HERESIS URITOR; DOCTRINA MANET**
