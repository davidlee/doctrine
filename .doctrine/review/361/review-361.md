# Review RV-361 — reconciliation of SL-251

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Mode: conformance.** Post-implementation audit of `SL-251`'s seven phases.

**Surface reviewed.** Not a dispatch candidate branch. `SL-251` was driven by
in-session capsule subagents on `refs/capsule/d/*`, landed as branch `sl-251`
(17 commits), and merged to `edge` at `eddb971c1` via the throwaway branch
`close/SL-251`. The audited surface is `edge` at `eddb971c1`. Runtime state
(`.doctrine/state/slice/251/`) and `.doctrine/slice/251/research/` are both
gitignored and existed only on `refs/capsule/d/state/implementation`; they were
restored from that ref before the audit ran, so the phase sheets and design run
are the capsule session's, not a reconstruction.

**Merge caveat carried into the audit.** `edge` and `sl-251` each allocated
backlog ids 434 and 366 from the same counter after the merge base, producing
four distinct items under two ids. The merge kept `edge`'s `IMP-434` and
`ISS-366` and re-minted the `sl-251` side as `IMP-438` and `ISS-439`. Anything
in this slice's record that names `IMP-434` before `eddb971c1` means `IMP-438`.

### Lines of attack

1. **Does the touch-set match the design?** `slice conformance` against
   `design.md`'s §7 table — every undeclared path is either scope creep or a
   design that stopped telling the truth, and every undelivered selector is
   dropped work or a stale declaration.
2. **Do the `VT` criteria actually verify what they claim?** `slice verify-vt`
   is a keyword grep over a named `test_file`; a PASS proves a keyword is
   present, not that the exit criterion holds, and a FAIL may be an artefact of
   where the implementation legitimately put the oracle. Read both directions.
3. **Are the corrections the phases promised to reconcile still owed?**
   `PHASE-07`'s `EX-10` names two design sections it says are wrong. Plan prose
   is not a reconciliation surface; the obligation has to reach the brief or it
   evaporates.
4. **Is the record of the slice true?** `notes.md`'s Harvest and Open sections,
   the recorded source-deltas, and the commit stream — a slice whose own notes
   describe a state it left days ago is not closeable.
5. **What rode in that was not the slice?** The capsule driver was itself new
   here; work its harness dropped into the tree is this slice's to disposition,
   not the next reader's to discover.

### Invariants held

- `ADR-001` module layering: leaf ← engine ← command, no cycles.
- `STD-001` no magic strings / single-source named constants.
- `POL-002` platform independence from host-project conventions.
- The behaviour-preservation gate: no wire type's definition or behaviour
  changes (`design.md` §7's own claim).
- `doctrine check gate` green on the merged tree.

## Synthesis

**The slice delivered what it set out to deliver, and the design stopped
tracking it in three places.** That is the whole closure story. Nine findings,
no blockers; not one of them says the code is wrong.

### The evidence

`doctrine check gate` is green on the merged tree. All seven phases report
`completed`. `doctrine slice verify-vt 251` is clean but for `PHASE-06/VT-1`,
which `F-1` establishes is a keyword-grep artefact over a correct oracle. `VA-1`
was discharged directly rather than read: `doctrine design contract` answers in
both formats from `/tmp`, outside any doctrine project, exit 0 — which is the
no-root property `sec-6` claims and a test inside the repo cannot see.
`PHASE-02/VT-2`'s waiver stands and is now load-bearing evidence for `F-2`.

`doctrine slice conformance 251` reports 10 conformant, 1 undelivered, 8
undeclared. Three of the undeclared are the id-collision residue (`F-6`,
tolerated); three more are the slice's own governance artefacts (`plan.md`,
`plan.toml`, `slice-251.toml`), which no design declares and none should. That
leaves exactly two real code paths — `src/commands/cli.rs` and
`src/design_run/artifact.rs` — and one real undelivered selector,
`src/design_run/attestation.rs`. Those three are `F-2`, `F-3` and `F-4`, and
they are the entire substance of the reconciliation.

### The shape of the drift

All three are the same failure, and it is worth naming because it is not
carelessness. In each case the implementation was **more right than the
design**, and the correction was made somewhere the design could not hear it:

- `attestation.rs` — `PHASE-02` discovered at execution that the file defines no
  closure struct and *waived the criterion with the reason written down*. The
  waiver is on a runtime phase sheet. `design.md` §7 still argues, at length,
  that omitting the file would be an error.
- `cli.rs` — `PHASE-07` discovered that `long_about` is inert because doctrine
  renders its own help, *bound the widening in a new exit criterion*, measured
  it across 281 renderings, and wrote down that `sec-6` and `sec-8` owe a
  correction. `EX-10` is in `plan.toml`, which is immutable-append and is not a
  reconcile write surface.
- `artifact.rs` — `PHASE-05` hoisted the repo-private-id detector to module
  level so both generated assets share one list, citing `STD-001` in the doc
  comment. Nothing recorded that the design's touch-set had grown.

So the durable lesson is about **where an execution-time correction lands**. A
waiver reason, a phase sheet and an appended `EX-` are all correct places to
record a discovery and all three are invisible to `design.md`. `plan.md`'s
per-phase execution headings were the slice's own answer to this and they worked
— every one of these was findable — but they are prose an auditor has to know to
read. The brief below is the first artefact in this slice's life that is *both*
enumerated and on a write surface.

### Standing risks, carried not closed

- **`sec-8` pin 1's premise is argued, not proven** — whether
  `assert_keys_described` stays one generic body across eleven types with
  different `Serialize` shapes. Recorded in `notes.md`; unchanged by this audit.
- **`RecordKind::ALL` is hand-maintained** and nothing forces a new variant into
  it. `sec-3` states the barrier at its real strength and `sec-8` pin 5 takes an
  exhaustive-match oracle instead, so the slice is sound over it. The knowledge
  tier's repair is `ISS-364`.
- **`ISS-362` is load-bearing on `sec-3`** — if a payload-act-to-stage guard
  lands as data, the stage column becomes derivable and `sec-3`'s subsection
  should be revisited.
- **The nominal-identity residue on `Named` edges** — `sec-8` pin 2 is total over
  today's closure only because no two targets present identically. The escalation
  (a `payload_struct!` mirroring `sec-4`'s enum instrument) is recorded and not
  taken.

### Tradeoffs consciously accepted

- `F-6` — three paths in an advisory report misattribute this slice's backlog
  item, because the delta registry stores oid ranges over a history that
  genuinely contains the pre-renumber paths. Rewriting `sl-251` is
  disproportionate; the mapping is disclosed in three places instead.
- `F-8` — a `skills-lock.json` hash this session cannot recompute stays at its
  refreshed value rather than being reverted to an equally unverifiable one.
- `F-9` — `slice selector doctor`'s advice on this slice is **not** taken.
  Following it would empty the conformance oracle for `src/design_run/`. `ISS-440`
  records the inversion.

### Merge-hygiene note for the next reader

This audit ran on `edge` at `eddb971c1`, which merged `sl-251` and resolved a
four-way backlog id collision by renumbering this slice's items to `IMP-438` and
`ISS-439`, and removed three duplicate agent definitions the capsule driver had
written to the repo root (`F-7`). Neither change is `SL-251`'s work; both are
recorded here because an auditor reading `sl-251` alone would find id references
that no longer resolve the way they did when they were written.

## Reconciliation Brief

Three findings reach a write surface. `F-1`, `F-5`, `F-6`, `F-7`, `F-8` and `F-9`
are terminal without a reconcile action — respectively adjudicated, already fixed
in the harvest tail, tolerated, already fixed in the merge, tolerated, and
captured as `ISS-440`.

Nothing here needs a REV: every target is a per-slice artefact. There are no
governance or spec changes owed.

### Per-slice (direct edit)

**`F-2` — `attestation.rs` is not a closure-struct home.** The load-bearing
change is the selector registry, which is what `slice conformance` reads:

- `doctrine slice selector rm 251 src/design_run/attestation.rs` — the
  `design-target` selector. While it stands, conformance reports this slice
  `undelivered` permanently.
- `design.md:1643` — delete the `src/design_run/attestation.rs` touch-set row.
- `design.md:1647-1654` — the prose immediately below the table argues that an
  earlier draft was wrong to omit `submission.rs` *and* `attestation.rs`. Correct
  it to `submission.rs` alone. `sec-8` pin 1's fixtures do sit beside each type's
  definition; the false step is that any closure struct is defined in
  `attestation.rs`. `PHASE-02/VT-2`'s waiver reason is the evidence:
  `CheckpointActDeclaration` is a `submission.rs` type at line 825.

**`F-3` — `cli.rs` and the help renderer.** `PHASE-07/EX-10` named this
correction and can only hand it over here:

- `doctrine slice selector add 251 src/commands/cli.rs --intent design-target`.
- `design.md` `sec-6:1518-1522` — the help-pointer mechanism reasons from clap's
  renderer. Doctrine's own `render_subcommand_help` (`cli.rs:1427`) reads
  `get_about()` only, and `main.rs:285-292` routes every `--help` through it, so
  clap's renderer never runs. State the shipped mechanism:
  `get_long_about().or_else(get_about())`, on the `or_else` shape already at
  `cli.rs:1319-1322`.
- `design.md` `sec-8` pin 7's help bullet — same correction, same reason.
- `design.md` §7 touch-set — add a `src/commands/cli.rs` row (~10 lines, the
  renderer fallback).

**`F-4` — `artifact.rs` and the shared detector.**

- `doctrine slice selector add 251 src/design_run/artifact.rs --intent design-target`.
- `design.md` §7 touch-set — add a `src/design_run/artifact.rs` row: hoist
  `REPO_PRIVATE_PREFIXES` and `cites_a_repo_private_id` from `mod tests` to
  module level as `#[cfg(test)] pub(crate)`, so `payload_contract`'s banner test
  shares one list (`STD-001`). All additions are `#[cfg(test)]`; no production
  path moves, so §7's behaviour-preservation claim is unaffected.

### Governance/spec (REV)

None.

### Explicitly off-surface

- **`plan.toml` is not edited.** `PHASE-06/VT-1`'s keyword set is imprecise
  (`F-1`) and `PHASE-07/EX-10` carries a correction obligation (`F-3`), but
  `EN-`/`EX-`/`VT-` ids are immutable-append. `F-1` is adjudicated on this ledger
  instead; `F-3`'s obligation is transcribed above.
- **`SL-251`'s selectors are not pruned on `slice selector doctor`'s advice**
  (`F-9`) — the two `add`s above are additive and deliberately leave the six
  flagged `design-target` selectors in place.

## Reconciliation Outcome

Every brief item is applied. No REV was authored, because no item targeted
governance or spec truth — all three reach per-slice artefacts only.

### Selector registry (the load-bearing change)

```
doctrine slice selector rm  251 src/design_run/attestation.rs                        # F-2
doctrine slice selector add 251 src/commands/cli.rs src/design_run/artifact.rs \
                                --intent design-target                               # F-3, F-4
```

`doctrine slice conformance 251` before / after:

| | before | after |
|---|---|---|
| undelivered | 1 (`src/design_run/attestation.rs`) | **0** |
| conformant | 10 | **12** |
| undeclared | 8 | 6 |

The six remaining undeclared are not code: three are the slice's own governance
artefacts (`plan.md`, `plan.toml`, `slice-251.toml`), which no design declares
and none should, and three are `F-6`'s tolerated id-collision residue under
`.doctrine/backlog/improvement/434/`. Every source path this slice touched is
now conformant.

### Direct edits applied

- **`design.md` §7 touch-set — row removed** (`F-2`): the
  `src/design_run/attestation.rs` row for "the closure structs defined here".
- **`design.md` §7 prose** (`F-2`): the paragraph arguing an earlier draft was
  wrong to omit `submission.rs` *and* `attestation.rs` is narrowed to
  `submission.rs`, with a `Corrected at reconcile` note recording that
  `PHASE-02` caught this at execution, waived `VT-2` on it, and that the waiver
  reason never reached canon.
- **`design.md` `sec-8` pin 1** (`F-2`, **second site, found while locating the
  first**): "Those fixtures live in `submission.rs` and `attestation.rs` … which
  is why `sec-7`'s touch-set lists both files as written" — a sentence that
  cross-references the very table above, so leaving it would have made the
  design internally inconsistent with its own correction. Verified empirically
  before rewriting: all **twelve** `fully_populated` fixtures are in
  `submission.rs` and none is in `attestation.rs`, whose contribution to the
  closure is five enums plus the newtype `ReviewRef(String)` — nothing with a
  key set, so nothing that takes an `assert_keys_described` call site.
- **`design.md` `sec-6` *Point 3*** (`F-3`): a `Corrected at reconcile` note
  under the two-altitudes claim, recording that clap's renderer never runs
  (`main.rs:285-292` → `render_subcommand_help`, `cli.rs:1427`, which read
  `get_about()` alone), that `long_about` occurred zero times in the tree so the
  mechanism was inert on arrival, and that `PHASE-07/EX-10` supplied the missing
  half.
- **`design.md` `sec-8` pin 7** (`F-3`): one appended paragraph — the pin's
  oracle is right, but it presumed `render_subcommand_help` surfaces
  `long_about`, which was a renderer change the design never scheduled.
- **`design.md` §7 touch-set — two rows added** (`F-3`, `F-4`):
  `src/commands/cli.rs` and `src/design_run/artifact.rs`.

`slice-251.md` needed no edit — scope did not change during implementation.

### REVs completed

None. No governance or spec item was raised.

### Withdrawn / tolerated / delegated

- `F-1` — `aligned`. `PHASE-06/VT-1`'s FAIL is a keyword-grep artefact over a
  correct oracle; adjudicated on this ledger because `plan.toml` is
  immutable-append and off-surface.
- `F-5` — `fix-now`, discharged in the audit's own harvest tail before this
  pass: `notes.md`'s Harvest and Open sections.
- `F-6` — `tolerated`. Unfixable short of rewriting `sl-251`; disclosed in three
  places instead.
- `F-7` — `fix-now`, discharged in the merge at `eddb971c1`.
- `F-8` — `tolerated`. The hash cannot be recomputed from the tree; reverting
  would trade a stale record for a churning one.
- `F-9` — `follow-up` → `ISS-440`. `SL-251`'s selectors are deliberately not
  pruned on that advice.

### Scope discipline

One thing was found during this pass that was not in the brief — the `sec-8`
pin 1 sentence above. It is **not** a new finding and no new finding was opened:
it is a second site of `F-2`, already dispositioned, and reconcile owns landing
a dispositioned finding at every site rather than at the one the auditor
happened to cite. Nothing else was re-audited.

Reconcile pass complete — handoff to `/close`.
