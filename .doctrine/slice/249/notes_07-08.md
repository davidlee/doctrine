# SL-249 — notes shard, phases 07–08

Execution record for this shard's phases. `notes.md` stays the orchestrator's.

> Shard name: the PHASE-08 sheet's T9 says `notes_08.md`; the orchestrator's
> brief says `notes_07-08.md`. Took the orchestrator's name (it owns the
> sharding scheme) — noted so nobody hunts for a missing file.

## PHASE-08 — the kind-blind `knowledge edit` verb

### T1 — share the body-flag helpers (D1)

Moved `resolve_body`, `parse_body_mode` and `BODY_MODE_REQUIRES_BODY` from
`src/memory.rs` to `src/input.rs` as `pub(crate)`, doc comments carried across.
`parse_body_mode` had **no** self-contained unit tests to move — its only
coverage is `memory edit`'s behaviour suite (`unknown body mode` asserted at
`src/memory.rs:8789` through the verb), so nothing moved with it.

Call sites updated: three production (`memory.rs` `run_record`'s body resolve,
`run_edit`'s mode + body resolve) and two test (`record_body_from_stdin`), plus
two `BODY_MODE_REQUIRES_BODY` test assertions. Call form is qualified
(`crate::input::…`) rather than a fresh `use`, so the module of origin reads at
the call site.

`tests/e2e_mcp_server.rs:1067`'s doc comment named `memory::BODY_MODE_REQUIRES_BODY`
as the const it deliberately mirrors; repointed at `input::`. The literal
duplication there is intentional (an integration crate cannot see `pub(crate)`)
and was left alone.

STOP-4 did not fire: `cargo test --bin doctrine memory` — 313 passed, no
assertion changed. Clippy workspace clean, `cargo fmt --check` clean.

### T2/T3/T5 — VT-1 and the verb

Red first: the two VT-1 tests failed to compile on `cannot find function
run_edit` / `cannot find struct EditFields`. Not committed red — this is the
primary worktree and another agent shares the tree, so a non-compiling commit
on `edge` is a cost borne by someone else. Red/green/refactor is preserved in
the working order, not in the commit boundary.

**The byte-equality oracle.** VT-1's "everything else unchanged" is asserted by
comparing the file text from `\n[facet]` to EOF, before and after. That tail
holds populated `[facet]` values, `[evidence]`, `[relationships]`, a
`[[relation]]` row and a hand-written comment. Comparing the *tail* rather than
diffing whole files is what makes the assertion legible: everything the verb is
allowed to write is a top-level scalar above it. Two fixtures — a facet-bearing
`DEC-007` and an empty-facet `CPT-003` (DEC-172) — cover both ends of the facet
spectrum with one code path and no kind dispatch, which is EX-5's claim.

**Signature.** `run_edit(path, reference, &EditFields, writer)` — a writer
param, not `run_status`'s direct `io::stdout()`, so the post-state print is
assertable. `EditFields` mirrors `memory::EditFields`. `root::find` returns an
explicit `path` verbatim, so tests drive the real verb against a temp root with
no marker file.

**`--body -` stdin is NOT reachable through the verb.** `run_edit` calls
`io::stdin()` internally, exactly as `memory::run_edit` does; the injectable
seam is `input::resolve_body(raw, &mut impl Read)`. So VT-2's stdin arm is
asserted on that helper with a `Cursor`, which is also how `memory.rs`'s own
`record_body_from_stdin` does it. Parity with the precedent was preferred over
threading a reader through a second verb's signature for one test.

**`write_record_body` gained `mode`** (D3), and the *"always `BodyMode::Replace`"*
sentence moved to `src/commands/design.rs`'s call site, where the reason
(a resumed step 5 re-applies the same payload and must produce the same bytes)
actually lives. Still exactly one path to a record's `.md` — I4's prose half.

**A fourth site the sheet did not name:** `src/commands/guard.rs:212` matches
`KnowledgeCommand` exhaustively for the read/write classification, so a new
variant is a compile error there until classified. `Edit` → `Write("knowledge
edit")`. Cheap, and the exhaustive match is the design working as intended —
worth knowing before adding any verb to a guarded command enum.

### T4/T6/T7 — body-mode parity, the guards, and the no-op

**Divergence, stated plainly: T4 and T6 had no red step.** The sheet ordered
T4 (red, VT-2) before T5 (green, the mode parameter), and T6 as red→green. In
practice T3's `run_edit` could not compile without `write_record_body`'s `mode`
parameter, so T5 landed with T3; and `run_edit`'s guards are part of the same
function, so T6's refusals existed before their tests. Both test sets therefore
passed on first run. That is a weaker signal than red/green, so the guard
assertion was checked with a **positive control**: swapping the totality guard
and the at-least-one-flag gate makes
`knowledge_edit_refusals_leave_both_tiers_byte_identical` fail, and restoring
the order makes it pass. The ordering assertion is not vacuous.

**VT-2's oracle is `entity::write_body`'s doc comment**, read line by line, not
recollection. Four arms plus vocabulary: `replace` overwrites; `append` onto a
body ending in one `\n` inserts exactly one blank line; `append` onto a body
already ending in `\n\n` does NOT double it; `  APPEND ` normalises; `prepend`
is a worded refusal. Nothing about prose semantics was decided here (STOP-3
never came close to firing).

**The no-op is asserted across all three concerns, not just the body** — same
title, a tag already present, byte-identical prose → no `write_atomic` on
either tier, both mtimes hold, `updated` still `2026-01-01`. The concept
fixture's `tags` are deliberately **unsorted** (`["zeta", "alpha"]`) so R2 is
exercised: `apply_tags_set` compares as sets, so an idempotent re-add against a
hand-authored store does not spuriously write. A separate test pins R1's other
end — a body-only edit DOES stamp `updated`, exactly once.

**A hazard found in `apply_tags_set` and designed around.** Its self-heal
(insert `tags = []` when absent) runs BEFORE its set-compare no-op guard, so a
call with an empty `adds` on a record missing `tags` mutates the held document
while returning `false`. On the "one open, one write" shape that means a
`tags = []` could ride out on the back of an unrelated concern's change.
`run_edit` sidesteps it by not calling the core at all when `adds` is empty.
Captured as `mem.fact.doctrine.apply-tags-set-self-heals-before-noop`.

### T8 — the diff read (EX-3, EX-5)

Base `a1c603bee` (this phase's first parent) to working tree. Six files:
`src/knowledge.rs`, `src/input.rs`, `src/memory.rs`, `src/commands/design.rs`,
`src/commands/guard.rs`, `tests/e2e_mcp_server.rs`.

Added lines were split at the `mod tests` boundary (`src/knowledge.rs:2101`)
so production and test adds are checked separately — 195 production lines.

- **EX-5 (no kind dispatch, no facet knowledge, no facet table).** Zero
  occurrences of any `RecordKind::<Variant>` in the production adds; zero
  `match kind`, `RecordFacet`, `validate_facet`, or any facet field name
  (`choice`, `rationale`, `consequences`, `decided_by`, `decided_on`,
  `alternatives`, `claim`, `verdict`); zero new `FACET`-ish const or table.
  **Positive control** (`mem_019fa18161f47651af7687d8dccbbc67` — a negative
  grep needs one): the same pattern matches **4** times across the full diff,
  all in the test fixtures (`RecordKind::Decision` / `::Concept` driving
  `seed_record`). The grep is live; the production count of 0 is a fact, not a
  broken pattern.
- **EX-3 (relations untouched).** Zero calls to `append_edge`, `remove_edge`,
  `apply_string_append`, and no write to `"supersedes"` / `"superseded_by"`
  anywhere in the diff. The only `relationships` / `facet` strings in the
  production adds are inside doc comments explaining why the verb stays off
  them. Backed by the stronger evidence: VT-1 asserts the file bytes from
  `[facet]` to EOF — populated facet values, `[evidence]`,
  `[relationships]`, the `[[relation]]` row and a hand-written comment — are
  identical before and after, on both fixtures. Asserted on bytes, never by
  reading an error.

### T9 — harvest

- `mem.pattern.doctrine.new-cli-variant-needs-guard-classification`
  (`mem_019fe0eacfd27693baf5960c7d639857`) — the `guard.rs` exhaustive match.
- `mem.fact.doctrine.apply-tags-set-self-heals-before-noop`
  (`mem_019fe0eafebb7ab1b74f02c2b43d6bbb`) — the self-heal-before-no-op hazard.
- friction `019fe0eb-21fc-7140-b39c-91627b691125` — the sheet's reading list
  omitted `guard.rs`.
- friction `019fe0eb-2203-7cf3-a2cc-d97ff3af170a` — the notes-shard name
  disagreed between the sheet and the spawning brief.

Nothing owed to the reconciliation brief from this phase beyond the T4/T6
red-step divergence above, which the orchestrator may want in `notes.md`'s
*Owed* ledger.

## PHASE-07 — the seven-kind amendment and its coverage canary

Movement 2 only (`T6`–`T9`, `[WORKER]`). Movement 1 (`T0`–`T5`, `[ORCH]`) —
`REV-050`, both tiers of `SPEC-019` and `PRD-010`, the `SPEC-004` anchor — was
already landed and green when this arm started; it is the orchestrator's record.

> Heading level: the sheet's `T9` says `### PHASE-07`. Used `##` so it sits as a
> sibling of `## PHASE-08` rather than nesting under that phase's `T9`. Same
> class of trivial divergence as the shard-name note at the top of this file.

### What was built

One integration test and its fixtures — `tests/governance_kind_coverage.rs`,
`tests/fixtures/governance_kind_coverage/{README.md,pre_amendment/*}`. **No
`src/` diff** across the whole movement (`EX-8`, `POL-002`): this is a
project-local test, never a `validate` rule. Nothing was added to `Cargo.toml`;
`regex` was already a normal dependency and `syn` already a dev-dependency, so
the no-new-dev-dependency constraint never came under pressure.

The checker is one `check(tiers, kinds, allow) -> Vec<String>` over five rules:

1. **strict-adjacency paired form** — each record kind must appear as
   `assumption (ASM)` (long name, optional backticks, parenthesised prefix)
   in each tier, not merely co-present in it;
2. a per-tier **`\bfour\b` arithmetic identity** against a `why`-bearing
   allowlist;
3. per-entry **exempt-phrase counts** (a stale exemption expires loudly);
4. the `why` argument is **emitted in the failure text**, so whoever trips an
   exemption reads the reasoning (`EX-5`);
5. an allowlist entry naming a tier not under test is itself a failure.

All measurement runs on **whitespace-collapsed** text.

### Census, before and after

| | `spec-019.toml` | `spec-019.md` | `spec-010.toml` | `spec-010.md` | total |
|---|---|---|---|---|---|
| `four`, pre | 3 | 25 | 0 | 4 | **32** |
| `four`, post | 0 | 1 | 0 | 0 | **1** |
| paired forms, pre | 0/7 | 7/7 | 0/7 | 1/7 | **8/28** |
| paired forms, post | 7/7 | 7/7 | 7/7 | 7/7 | **28/28** |

The surviving post-amendment `four` is the exempt phrase. Both censuses were
re-derived from the tree (`git show 59f77ce10:<path>` for the pre side) rather
than copied; both reproduced the sheet exactly. **`R-inventory` did not fire on
the counts** — it fired instead on a list the canary cannot see (below).

### The collapse is load-bearing, and the sheet's measure needed correcting

Two things worth carrying past this phase.

**Strict adjacency, not co-presence.** The sheet's `T6` text specified a
co-presence measure. Co-presence scores the *pre-amendment* `spec-019.toml`
4/7 when its true paired-form score is **0/7** — the file names the four old
kinds and their prefixes in separate sentences and never pairs them. A measure
that reports 4/7 for a file with zero pairings is not measuring the claim; it
would have made the compensating control assert a fiction. Corrected to strict
adjacency before the fixtures were written.

**Whitespace collapse.** `spec-019.md` wraps the exempt phrase mid-phrase — one
line ends `…touches four`, the next begins `coupled sites — `. Raw, the phrase
matches **zero** times; collapsed, **once**. Without the collapse the identity
reads `32 == 0` and the only green path is rewording an accurate sentence to
satisfy a test. A fixture asserts **both** directions against those exact bytes,
and the fixture README says not to reflow them.

### The pre-amendment control and its exact failure set

The live arm was written *after* the corpus was corrected, so it passed on its
first run and proves nothing alone — a checker returning `vec![]`
unconditionally passes it too. The compensating control runs the **same**
`check` over the pre-amendment bytes, recovered with `git show 59f77ce10:<path>`
(no branch switch), and asserts the failure set is **exactly 23**:

- **20 missing paired forms** — 7 + 7 in the two TOML tiers, 3 each in the two
  prose excerpts;
- **3 identity failures** — one per tier carrying an unaccounted `four`;
- **0 exempt-phrase failures** — the exemption was already correct pre-amendment.

Exactness is the point: a count assertion would pass for the wrong reasons.

The `.md` tiers are ~60 KB combined, so those two fixtures are **excerpts**, and
the excerpt's own completeness is asserted (`spec-019.md.excerpt` carries all 25
`four`, `spec-010.md.excerpt` all 4) — a short excerpt would quietly make the
control a lie.

A positive control guards the live arm too: `the_live_arm_notices_a_wrong_allowlist`
perturbs the allowlist against the **live** tiers with the **full parsed** kind
list and asserts **exactly two** failures, which simultaneously re-asserts that
all 28 pairings pass and the other three tiers are clean.

### The kind list comes from parsing source, and why it must

`kinds::RECORD` is `pub(crate)` and nothing is exported from `src/lib.rs`;
widening it would breach `tests/architecture_layering.rs`. But export policy is
not actually the reason — **the long-name↔prefix pairing exists nowhere but the
const identifiers** (`ASSUMPTION_KIND` ↔ `prefix: ASM`), so even a `#[path]`
include could not supply it. Parsing is required, not merely the lesser evil.

`record_kinds_from_source()` runs two independent `syn` parses — the
`stem: "record"` `Kind` consts (the pairing) and the `RECORD` slice at
`src/kinds/mod.rs:57` (the membership list) — and **panics naming both** if they
disagree. Prefixes resolve through the file's own `&str` consts rather than
being read off the identifier text, so `prefix: ASM` is *verified* to be `"ASM"`.

Fixture arms deliberately use a **frozen** seven-kind list, not the parsed one,
so an eighth kind cannot retroactively rewrite the historical control's
arithmetic; `frozen_kinds_are_still_real_kinds` catches renames and removals.

`LIVE_TIERS` is a **path-listed** const of four `(name, path)` pairs, never a
glob (`D9`): `.doctrine/spec/tech/019/` also holds `handover.md`, gitignored
scratch that still carries the stale four-kind claim *by design*. If the canary
ever sees `handover.md`, the scope is wrong.

11 tests, green.

### `VA-1` — the agent read (`T8`)

Read by the worker, not the prose author. **The amendment is sound and the
`ISS-316` boundary HELD** — no EVD/HYP/CPT status vocabulary and no supersession
row for the three, despite all three `*_STATUSES` consts having shipped, which
is exactly `F-13`'s temptation. `spec-019.toml`'s lifecycle-vocabulary and
supersession responsibility rows are byte-identical across `59f77ce10..HEAD`
(confirmed by diff). The one change inside the supersession section is
`four` → `seven` in the `TargetSpec` sentence: factual de-staling of
`TargetSpec::Kinds(RECORD)` (`src/relation.rs:457–466`), not a new rule.

Facet contracts match `src/knowledge.rs` field-for-field, including concept's
deliberate emptiness (named as a *case, not an exception*, with `edit concept`
refusing). Verb set names what shipped.

**One defect, in the amendment's own new material** — `ISS-332`. `provenance` is
a fourth closed enum, introduced by this amendment, and two sentences
enumerating the closed enums were not swept (`spec-019.md` ~l.128, and ~l.353's
*Facet-enum drift* risk). Prose-only: `provenance` does ride the `"" -> None`
seam and does have the known-set guard the risk asks for. This is the phase's
fifth `R-inventory` firing, in a class **the canary is structurally blind to** —
a stale facet-enum list carries no kind name, no prefix and no numeral. Worth
stating plainly for reconcile: the canary narrows the hand-maintained-list
class, it does not close it.

### Divergences from the sheet

1. `T6`'s measure changed from co-presence to **strict adjacency** (above) — the
   sheet's own `F-5` failure mode.
2. `T6` fixture 2 as written does not demonstrate what `F-11` bought: a
   single-tier duplicate fails *both* halves of the exempt-phrase rule. Added a
   second arm using **two** allowlist entries so the total balances (2 == 2) and
   only the per-entry half can fire.
3. Added a fifth rule the sheet did not ask for — an allowlist entry naming a
   tier not under test is a failure. Cheap; the alternative is a silently
   ignored exemption.
4. `T9` heading level (above).

### Harvest

- `mem.pattern.doctrine.conformance-measure-must-match-the-claim`
  (`mem_019fe22b652973239aa273bef6ace8ca`) — a conformance measure that is
  *looser* than the claim scores a defective corpus as partially compliant; pick
  the measure against the pre-fix bytes, not the fixed ones.
- friction `019fe22e-bb78-7b43-8174-5b6455b54221` — a stub-based red costs a
  **compile** cycle in this crate: `-D unused` turns the stub's unread fields
  and helpers into hard errors and `-D elided-lifetimes-in-paths` rejects
  `&[Tier]`. A sheet prescribing a stub-based red in a deny-unused crate should
  say to seed the temporary `dead_code` allow up front.
- `ISS-332` (open) — the `provenance` omission, plus a legibility note in its
  tail: three `SPEC-019` sections are knowingly four-kind and say nothing about
  `ISS-316` owning the rest, so a cold reader cannot tell *scoped* from *stale*.
  Deliberately not fixed here — widening `EX-6` is the `F-13` failure.

Owed to reconcile: divergences 1–3 above, and the canary's blind spot named in
`ISS-332`.
