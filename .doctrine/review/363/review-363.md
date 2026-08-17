# Review RV-363 — reconciliation of SL-238

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** The primary worktree on `edge` at `674d5b284`, not a
dispatch candidate branch — SL-238 ran solo in-tree across eight phases, so
`review/*` and `phase/*` evidence refs do not apply and there is no candidate
interaction branch to admit.

**What this audit probes.** SL-238's thesis is that a surface must not state
what it has not checked — the footer's `dropped (dangling: SL-182 absent)` lie
is the slice's founding defect. An audit of that slice earns no credit for
re-reading its own records, so every claim below was re-derived against the
built tree or the live corpus.

Lines of attack:

1. **The disclosure is true of real data.** Enumerate the authored cross-kind
   `needs`/`after` edges directly from the backlog tomls, classify each
   dependent and target by status, and check the rendered `boundary:` block is
   exactly the non-terminal-target, non-terminal-dependent set — no more, no
   fewer. A footer that suppresses a live edge repeats the founding defect in
   the other direction.
2. **The two new surfaces agree.** `doctor`'s ref-integrity leg and the
   `backlog list` stderr advisory both count unresolvable refs. On a clean
   corpus both must be silent, and their silence must be a positive control
   rather than a dead code path.
3. **The layering result is measured, not asserted.** `DEC-231`'s repair was
   dependency inversion over relocation precisely because the tangle is
   measured. Run the suite; read the baseline; confirm `src/backlog.rs` reaches
   `crate::commands` nowhere in production, with a positive control.
4. **The deletions are deletions.** `classify_dangling`, the `Dangling` render
   arm, `render_overrides`' `corpus` parameter, `title_for`, `backlog.rs`'s
   duplicate prune leg — each must be gone from the tree, not shadowed.
5. **No suite was edited to pass.** `src/meta.rs`, `src/backlog_order.rs` and
   the `search`/`map`/`catalog` and `priority` suites are declared unmodified
   (PHASE-01 `EX-8`/`EX-9`, PHASE-04 `EX-8`). Verified by diffing the whole
   slice span for *removed* assertions, not by trusting the phase records.
6. **The evidence apparatus is not self-certifying.** `slice verify-vt` reports
   48/48 `PASS`; `ISS-441` is open against exactly that signal. Re-derive the
   rows whose keywords are common words or name superseded tests.
7. **Every design/plan divergence the phases recorded is real and routed.**
   `notes.md ### Open` carries eleven; confirm each, find what it missed, and
   map every one to a write surface `/reconcile` will actually touch.

**Invariants held.** `STD-003` (tolerate *and* disclose — no laundered read, no
silent skip); `STD-001` (one terminal-status table, reached through
`partition::status_class`); `ADR-001` (no new command-tier cycle, tangle
baseline 76); `ADR-017` (`authored_class` sits beside the classifier it
extends); `ADR-009` (slice terminal is `done`, not backlog vocabulary);
`POL-002` (no host-project convention in the engine).

**Where the bodies were likely buried.** Three places, from the slice's own
history: the design's prose claiming a census it did not re-run (five separate
instances found during execution); a criterion stated in prose that no
verification row can carry (`PHASE-05 EX-2`); and the conformance registry,
which is hand-maintained and therefore drifts silently from what the slice
actually touched.

## Synthesis

### The closure story

SL-238 began with a surface that lied. `backlog list --by sequence` trailed
lines of the form `ISS-028 → SL-182 dropped (dangling: SL-182 absent)`, and
every one of them was false — `SL-182` exists, and always did. The footer had
hardcoded the word `absent` for a case its resolver could not name, so a
cross-kind reference became, in the rendering, a missing entity. An interim
commit silenced those lines, which removed the lie without supplying the truth.

Eight phases later the same command prints ten lines, and I checked all ten
against the corpus rather than against the slice's records. Enumerating the
authored `needs`/`after` refs straight out of the backlog tomls gives 32
cross-kind edges. Classify each dependent and each target by status and exactly
ten survive both filters — a non-terminal dependent (the projection admits no
other) and a non-terminal target (a satisfied prerequisite explains nothing).
Those ten are the ten rendered, in that order, with those statuses. The
thirteen suppressed targets — `SL-095`, `SL-138`, `SL-147`, `SL-154`, `SL-170`,
`SL-176`, `SL-182`, `SL-189`, `SL-190`, `SL-228`, `SL-250`, `SL-251` and
`QUE-212` — are every one of them `done` or `answered`, checked individually.
The footer now states what it has checked, which was the whole point.

The other two surfaces agree with it. `doctrine doctor` raises no
`RelationIntegrity` finding from the new `dep_seq_ref_findings` leg, and
`backlog list` emits no stderr advisory: the corpus has zero unresolvable
authored refs, and the two independent counts of that population both say zero.
That agreement is worth more than either number alone — it is the positive
control that separates a working check from a dead code path, and it is the
shape the slice's own §2 argued for.

The layering result held under measurement rather than assertion, which is the
part that could most easily have gone wrong. `RV-358` `F-1` was raised as a
blocker at design: the first repair for `backlog`'s need to reach three
`commands::dep_seq` operations would have closed a new command-tier cycle, and
the second (relocating them to an engine-tier module) would have created an
upward edge — worse. The design landed on dependency inversion instead:
`cli.rs`, which already depends on `backlog` downward, fills a `DepSeqOps`
struct of `fn` pointers. At audit, `src/backlog.rs` names `crate::commands`
nowhere in production code — the four occurrences are inside a `#[cfg(test)]`
module, which `tests/architecture_layering.rs` provably excludes (it carries
`extract_edges_excludes_cfg_test_mod_items` as its own proof) — and the command
tangle measures 76, the baseline unchanged. The suite is green.

The deletions are deletions. `classify_dangling`, `render_overrides`' `corpus`
parameter, the `AbsentDrop` render leg, `catalog::scan::title_for`, and
`src/backlog.rs`'s duplicate prune leg are all gone from the tree. No terminal
status literal survives in `src/commands/dep_seq.rs` and no laundered
`unwrap_or_default()` read does either — both established by grep with a
positive control, because a negative grep result without one is not evidence.
And no suite was edited to pass: diffing the whole slice span for *removed*
lines shows `src/meta.rs` and `src/backlog_order.rs` untouched, and not one
existing assertion deleted from `src/catalog/scan.rs` or
`src/priority/partition.rs` — only additions.

Ten findings, none a blocker. Two fixed in the audit, two accepted as drift with
their owners named, one confirmed as a false alarm, five routed to reconcile.

### Standing risks

**The evidence apparatus is partly self-certifying, and this slice found the
seam.** `slice verify-vt` reports 48/48 `PASS`, and three of those rows
(`F-2`) name characterisation tests that were deliberately retired and no longer
exist — their names survive only inside the doc comments of the tests that
superseded them, because the plan *required* each replacement to say what it
replaced. Good discipline manufacturing a false positive is a nastier failure
than careless keywording, and `ISS-441`'s two leading candidate fixes both miss
it. Nothing in SL-238 is unpinned; the risk is to the next slice that reads a
green summary as evidence.

**The conformance registry is hand-maintained and drifted silently** (`F-1`).
The one source file it missed is the one holding fourteen of the forty-eight VT
rows — the black-box golden three phases *retargeted to* precisely because the
assertions were over rendered CLI output. A registry that is only as good as
the discipline of the agent updating it will keep producing findings whose
signal is indistinguishable from real scope creep.

**Five design defects at one seam, and the seam is still open.** Every one of
`F-6`'s eight items was found by a phase-planning seat that re-derived a claim
instead of transcribing it, and five of them are the same shape: a prose rule
and the criterion resting on it agreeing with each other and disagreeing with
the tree. `DEC-242`'s arrangement — a test-authoring seat blind to the type
prototype — is what surfaced most of them, which is evidence for the
arrangement and not against it. But the design run locks, and a locked design
accumulates falsified prose for the whole of execution. The eight corrections
below are the interest payment.

**A shipped memory recommended a rejected repair, at high trust, for this exact
seam.** `mem.fact.layering.gate-measures-top-level-modules` closed its "the
repair that works" section by citing `src/dep_seq_ops.rs` — a module that never
existed, because it is the alternative `RV-358` `F-1` killed. It was corrected
at PHASE-08 harvest and I re-read it to confirm: `dep_seq_ops` now appears only
as the named rejected alternative, and the injection idiom is cited as what the
slice actually did. Worth recording because the corpus has no reconcile pass —
a memory written mid-slice against a decision that later reverses has nothing
to catch it, and this is the first instance found in the memory corpus rather
than in `design.md`.

### Tradeoffs consciously accepted

**The advisory and `doctor` do not count the same population, and closing the
gap was refused.** `project` iterates only non-terminal items, so a broken ref
authored on a terminal dependent never reaches the advisory; and the listing
path's fail-fast `read_all` has no counterpart to `dep_seq_ref_findings`'
`ReadFailure` disclosure. Closing either would require the second corpus
traversal §4 explicitly forbids. What holds is parity over the three
broken-ref *classes*, which is what the tests assert. This is a priced tradeoff
and `F-6` item 3 makes the design say so.

**A status-less target became prunable, and the class is rendered rather than
invented.** `authored_class(kind, Absent)` is `Terminal`, so an `after` edge
onto a `REC` now prunes where today it is kept. There is no status word to
render, so the reason names the class: `dropped (dangling: status-less)`.
Treating `Absent` as *keep* was the alternative and was refused because it
contradicts `status_class`'s own documented meaning. Reachability is low —
`REC` is not an admissible `after` target, so only a hand-authored edge gets
there — but that is exactly the population a repair verb exists to serve.

**Three author-time guarantees were given up on the remove path, deliberately.**
`after --remove` and `needs --remove` now gate the SOURCE only, surrendering the
target's on-disk resolution, its kind gate and the self-edge refusal. Without
that, the refs `doctor` reports at Error severity would be precisely the refs
`--remove` cannot touch — a checker that names damage the tool refuses to
repair. The authoring verbs keep the full gate.

**One known bound is asserted rather than fixed.** A stored `needs = ["SL-1"]`
is not cleared by `--remove SL-1`, because the needle canonicalises to
`SL-001`. Pinned as a test so closing it later has to be deliberate.

**`backlog needs` now refuses input it accepted yesterday.** `RV`, `REC` and
governance targets are rejected with the byte-identical message the kind-neutral
verb gives (`ISS-368`, PHASE-08 `EX-4`). Correct, and still a compatibility
break — it is on the named-output-change list for that reason.

### On the design's own numbers

§1 records 31 authored cross-kind edges, 26 reaching the footer, 11 with a
non-terminal target. At audit: 32 edges, 10 boundary rows. The design says in
terms that these are a dated snapshot of a moving population and that nothing
depends on them holding, and the drift is fully explained — `ISS-367 after
SL-256` was authored after the count, and targets went terminal in between
(`QUE-221` was answered on 2026-08-17, mid-slice). No correction is owed. I
record it only because a future reader comparing the two numbers deserves to
find the reconciliation already done.

## Reconciliation Brief

### Per-slice (direct edit)

- **`F-1` — the selector registry, then its mirror.** Run `doctrine slice
  selector add 238 tests/e2e_dep_seq_verbs.rs --intent design-target`. That
  registry (`slice-238.toml`) is what `slice conformance` reads; a prose-only
  edit leaves the delta red. Then add the same row to `design.md` §8's change
  table as the human mirror. The file carries PHASE-02 `VT-1`…`VT-3`, PHASE-07
  `VT-1`…`VT-5` and PHASE-08 `VT-1`…`VT-3`.
- **`F-6` item 1 — `design.md` §3 `The three standing rules the degradation
  carries` (rule 2) and §7 `The probe` (the `VT-5` bullet).** Both claim a
  future derived-status kind *omitted* from `DERIVED_STATUS` "must fail a test,
  not degrade quietly". An equality pin fires on addition, never omission.
  State what the pin does — a tripwire on intent — and name the strict
  `meta::read_meta` failure as what actually enforces completeness. Owner
  accepted 2026-08-16.
- **`F-6` item 2 — `design.md` §7 `The probe`, the `VT-2` bullet.** Re-fixture
  it from "a derived-status kind with no toml at all" to "a derived-status kind
  whose toml does carry a status", and say that "without reading" names the
  *status* read, never the title read. §3's arm table already decides this;
  `plan.toml`'s `VT-2` is already amended. Owner accepted 2026-08-16.
- **`F-6` item 3 — `design.md` §2's parity sentence** (`:373-374`). Narrow
  "the advisory's count and §5's check report the same population" to the
  class claim, and name the two accepted gaps: terminal dependents (which
  `project` never admits) and unreadable items (fail-fast `read_all` on the
  listing path). Both are costs of the single-walk constraint §4 imposes.
  Unconditional.
- **`F-6` item 4 — `design.md:1187`.** "Three call sites move to the new form"
  is four; the fourth is `run_after_prune`'s removal loop, which the signature
  reshape breaks at compile time. Name it and note PHASE-07 rewrites it a
  second time. `plan.toml` `EX-1` already amended.
- **`F-6` item 5 — `design.md:1487` (§6) and `:1665` (§7).** `--prune` did have
  coverage: five SL-105-era goldens. Correct both sentences to say those
  goldens pin the *decision* (which edges survive) and not the *rendered
  reason*, and add the five to §7 `Preservation` with `after_prune_absent_target`
  marked superseded by PHASE-07.
- **`F-6` item 6 — `design.md` §6's opening sentence and §7's agent-verified
  bullet.** "Four hardcoded copies of `status == resolved || closed`" conflates
  two populations: there are **two** literal comparisons and **four**
  read-parse blocks, which §6's next paragraph already states correctly. Fix
  the label and the four stale line numbers (`backlog.rs:2019`/`:2043` →
  `:2295`/`:2319`; `commands/dep_seq.rs:202`/`:222` → `:287`/`:307`).
  `plan.toml`'s PHASE-07 objective and `EX-3` already amended.
- **`F-6` item 7 — `design.md` §6 gains a fifth `--prune` consequence.** An
  `after` edge onto a `REC` becomes prunable, rendering `dropped (dangling:
  status-less)`. Owner accepted 2026-08-17; `plan.toml` `EX-4`/`VT-5` already
  amended.
- **`F-6` item 8 — `design.md` §5, optional.** Either note that a non-canonical
  stored ref reports under `not a canonical ref` regardless of *why* it failed
  (a dangling **bare** ref is the case), or decline as immaterial — measured
  zero bare refs in the corpus. Owner's call; no `VT` row depends on it.
- **`F-7` — record PHASE-05 `EX-2`'s disposition in `design.md` §7 and
  `notes.md`.** Recommended: accept `EX-2` as agent-verified, citing the two
  mitigations that landed (the Table arm's single inline expression, and
  `VT-3`'s doc comment stating what it does not prove). **Do not edit
  `plan.toml`** — `EN-/EX-/VT-` ids are immutable-append and are not a
  reconcile write surface.
- **`F-8` — the named-output-change list** belongs in the reconciliation record,
  not only on this ledger. **Eight** entries: the `AbsentDrop` footer leg
  deleted; the `boundary:` block introduced; the count-only stderr advisory
  introduced; `--prune`'s three reason strings collapsed to `dropped (dangling:
  unresolved)` with the `/resolution` suffix dropped; the fifth token `dropped
  (dangling: status-less)`; the new `--prune` stderr line `{source_id} after
  {to} (rank {r}) kept (unreadable: {err:#})`, which PHASE-07 `EX-3` mandated
  without specifying wording; `after --prune`'s source echo becoming canonical
  (a change to the **top-level** verb, which PHASE-08 `EX-6` covers only for
  the routed legs); and `backlog needs` refusing `RV`/`REC`/governance targets.

  `F-8`'s response on the ledger says *seven* and omits the `kept (unreadable:
  …)` line. It was undercounted there and corrected here, caught by the audit's
  phase-sheet sweep — PHASE-07's sheet flags it as "a sixth output change for
  the reconciliation brief" and `notes.md:630` carries it. The ledger is
  append-only, so the count stands there and the brief is the authority.

### Governance/spec (REV)

None. SL-238 changed no ADR, policy, standard or spec: `ADR-001`'s layering
table gained one authored classification row (`authored_status = "engine"`)
under its existing rules with the tangle baseline unmoved, which is data the
ADR's own mechanism expects, not a governance amendment. `ADR-017`, `STD-001`
and `STD-003` were all satisfied as written.

### Intake dispositions (drive at reconcile/close)

- **`IMP-099`** (`triaged`) — the `fulfils` edge is already authored in
  `slice-238.toml`; transition it on the fulfilment burndown (`ADR-018`).
- **`IDE-019`** (`open`) — transition as fulfilled, **and** write its two
  declined mechanisms into `IDE-019`'s own body, because its proposer reads
  `IDE-019` and not this slice's notes: ref-integrity went to `doctrine doctor`
  rather than behind a footer flag (`DEC-232`), and per-item detail went to
  `backlog inspect`/`show` rather than behind a listing flag (`DEC-234`). The
  intent was delivered; the mechanism was declined twice.
- **`QUE-222`** (`open`) — settle it (`doctrine knowledge settle`). Answer it
  together with `F-4`: the advisory template hardcodes the plural and renders
  "1 … refs name nothing", and both the plural and the completeness wording are
  the same one-line edit. Three tests pin the count.
- **`DEC-236`** — confirmed at audit as covered by `design.md` §9's overrun
  fold, not an uncovered divergence. No action.

### Already discharged in this audit

- `F-3` — `render_overrides` now composes the shared `annotated` guard; doc
  comment corrected. Green.
- `F-5` — `CHR-071` minted and linked (`references SL-238 --role
  originates_from`).
- `F-2` — `ISS-441` extended with the second route, its measurements, and a
  fourth candidate fix.
- `RV-358` — the design ledger's five `answered` findings, including the `F-1`
  blocker, verified terminal against the built tree. `RV-358` is `done`.

## Reconciliation Outcome

Written 2026-08-17 against head `ae2e1f195`. Every brief item is resolved; nothing
is escalated to design and no REV is owed. Findings keep their `verified`
disposition — remediation is recorded here, never by mutating a finding.

### Direct edits applied

**The selector registry, and its mirror** (`F-1`). `doctrine slice selector add 238
tests/e2e_dep_seq_verbs.rs --intent design-target` — the registry in
`slice-238.toml` is what `slice conformance` reads, and it now reports the file
**conformant** rather than undeclared, with `undelivered` still 0. The 135
remaining undeclared paths are authored `.doctrine/` entities, memories and
observation records: expected noise, no source among them. `design.md` §8 gains
the matching row in the change table *and* in the design-target selector block —
the mirror, not the fix.

**`design.md`, eight prose corrections** (`F-6`). Each edit carries an inline
`RV-363 F-6 item N` attribution so a future reader finds the reconciliation
without leaving the artefact.

| item | section | correction |
|---|---|---|
| 1 | §3 rule 2, §7 `The probe` | The `DERIVED_STATUS` equality pin is a tripwire on *intent*; it fires on addition and never on omission. `meta::read_meta`'s strict read under STD-003 is what actually protects an omitted kind. Owner accepted 2026-08-16. |
| 2 | §7 `The probe`, `VT-2` | Re-fixtured to a derived-status kind **whose toml carries a status**; *"without reading"* names the status read, never the title read. §3's arm table decides it. Owner accepted 2026-08-16. |
| 3 | §2 parity sentence | Narrowed from population parity to **class** parity, naming both accepted gaps — terminal dependents (`project` never admits them) and unreadable items (fail-fast `read_all`) — as priced costs of §4's single-walk constraint. §5's mirror sentence narrowed to *counting convention* in the same pass, so the two sections no longer disagree. Unconditional. |
| 4 | §6 (`:1187`) | Three call sites → **four**; the fourth is `run_after_prune`'s removal loop, broken at compile time by the signature reshape and rewritten a second time by PHASE-07. Stale line numbers dropped rather than refreshed. |
| 5 | §6 (`:1487`), §7 (`:1665`), §7 `Preservation` | `--prune` **had** coverage: five SL-105-era goldens. Both sentences now say those goldens pin the *decision*, not the *rendered reason*. The five are added to `Preservation` with `after_prune_absent_target` marked superseded by PHASE-07 — the supersession the design should have declared. |
| 6 | §6 opening, §7 agent-verified | **Two** literal terminal comparisons inside **four** read-parse blocks, which §6's next paragraph already said correctly. Four stale line numbers restated (`backlog.rs:2295`/`:2319`; `commands/dep_seq.rs:287`/`:307`). |
| 7 | §6 | Gains the **fifth** `--prune` consequence: `authored_class(kind, Absent)` is `Terminal`, so an `after` edge onto a `REC` becomes prunable, rendering `dropped (dangling: status-less)`. The refused alternative (treat `Absent` as *keep*) is recorded with its reason. Owner accepted 2026-08-17. |
| 8 | §5 | **Owner's call taken: note it.** A non-canonical stored ref reports under `not a canonical ref` regardless of why it failed; a dangling **bare** ref is that case. Measured zero bare refs in the corpus, so exhaustive in practice, imprecise in principle. |

Six of the eight had their corresponding `plan.toml` criterion amended in place at
planning time; those two artefacts now agree again.

**PHASE-05 `EX-2` accepted as agent-verified** (`F-7`). Recorded in `design.md` §7
`Agent-verified` and pointed to from `notes.md`. The e2e golden was declined —
`tests/e2e_inspect_golden.rs` is about `doctrine inspect`, not `backlog inspect`,
so pinning three lines of wiring costs a new e2e file — and the acceptance rests on
the two mitigations that landed at PHASE-05 (the `Table` arm's single inline
expression; `VT-3`'s doc comment stating what it does not prove). **`plan.toml` was
not edited**: `EN-`/`EX-`/`VT-` ids are immutable-append and are not a reconcile
write surface.

**`notes.md` `### Open` closed.** Every entry now carries a disposition in a
summary table at the head of the section, with the evidence trail left intact
below it. `A2` is recorded as **deliberately left unverified** — no code, criterion
or prose branches on the answer, so the census would change nothing.

### Code change

**The unresolvable-refs advisory: softened and pluralised** — `QUE-222` settled
`answered` in favour of softening, together with `F-4`, because both defects rode
one string.

```
backlog list: at least {n} authored needs/after {ref names|refs name} nothing — run `doctrine doctor` for the full check
```

*At least* declares the count a lower bound; *for the full check* declares the
pointer complete. This is §2's own argument turned on the advisory — a signpost
trusted as complete when it is not is worse than no signpost — and the two gaps
that make it incomplete are the ones §2's narrowing now names. Shape: one template
plus two agreement constants (`STD-001`), TDD red→green. **Correction to the
brief's arithmetic:** the count was pinned in **four** `contains` assertions
(`src/backlog.rs:6255`, `:6292`, `:6328`, `:6420`), not three — both `F-4` and
`QUE-222` estimated three. The *"at least"* prefix broke none of them; the singular
broke the two at `n = 1`. `design.md` §2's rendered example updated to match.

### Intake dispositions

- **`IMP-099`** → `resolved · done`. Fulfilment burndown on the `fulfils` edge
  already authored in `slice-238.toml` (`ADR-018`; no degree recorded, so `None ≡
  Full`).
- **`IDE-019`** → `resolved · done`, **and its body now records both declined
  mechanisms**, because its proposer reads `IDE-019` and not this slice's notes:
  the `--verbose`/`--explain` flag was declined (`DEC-234` — per-item detail went
  to `backlog inspect`/`show`), and the dangling-ref report was **resited rather
  than gated** (`DEC-232` — authored data that is wrong is `doctor`'s business).
  The item's two open questions are answered by that siting. Intent delivered,
  mechanism declined twice.
- **`QUE-222`** → `answered`, disposition on the record, prose half in
  `record-222.md`.
- **`DEC-236`** — no action, as the brief found: covered by §9's overrun fold.

### The named output changes — eight (`F-8`)

The authoritative list. `F-8`'s response on the ledger says *seven* and omits
entry 6; the ledger is append-only, so the count stands there and this is the
correction.

1. **`dropped (… absent)` no longer renders anywhere.** The footer's project-level
   `AbsentDrop` leg is deleted and `classify_dangling` with it. This was the
   slice's founding defect — the word `absent` hardcoded for a case the resolver
   could not name.
2. **A new `boundary:` block**, dependent-first, no arrow. Ten lines on the live
   corpus, each one an edge whose dependent *and* target are both non-terminal.
3. **A new count-only stderr advisory** on `backlog list --by sequence`, silent on
   the live corpus (zero unresolvable refs). Wording as settled above.
4. **`after --prune`'s three reason strings collapse to one** — `absent`,
   `(unparseable)` and `absent (unparseable ref)` all become `dropped (dangling:
   unresolved)` — and the terminal reason **loses its `/resolution` suffix**
   (`dropped (dangling: closed)`, not `closed/wont-do`).
5. **A fifth reason token, `dropped (dangling: status-less)`**, for an `after` edge
   onto a `REC`.
6. **A new `--prune` stderr line**, `{source_id} after {to} (rank {r}) kept
   (unreadable: {err:#})`. PHASE-07 `EX-3` mandated the disclosure without
   specifying wording. *This is the entry the ledger's response undercounted;*
   PHASE-07's phase sheet flagged it as "a sixth output change for the
   reconciliation brief" and `notes.md:630` carries it.
7. **`after --prune`'s source echo becomes canonical** where it was as-typed. Note
   for release notes: this is a change to the **top-level** verb, which PHASE-08
   `EX-6` covers only for the routed `backlog after` legs.
8. **`backlog needs` refuses `RV`, `REC` and governance targets**, with the
   byte-identical message the kind-neutral verb gives (`ISS-368`, PHASE-08 `EX-4`).
   Correct, and a **compatibility break** — input accepted yesterday is refused
   today.

### Governance/spec (REV)

**None owed, as briefed.** `SL-238` amended no ADR, policy, standard or spec.
`ADR-001`'s layering table gained one authored classification row
(`authored_status = "engine"`) under its own existing mechanism with the tangle
baseline unmoved — data the ADR expects, not a governance amendment. `ADR-017`,
`STD-001` and `STD-003` were satisfied as written.

### Carried, not fixed here

- `F-2` / **`ISS-441`** (`open`) — `verify-vt`'s false `PASS`. Extended at audit
  with the second route, its measurements and a fourth candidate fix. The defect is
  in the evidence tool, not this slice.
- `F-5` / **`CHR-071`** (`open`) — the `kref_for` collapse, minted at audit and
  linked `originates_from SL-238`.
- `F-10` — confirmed correct as-is; no criterion was lost. No action.
- **`RSK-013`** and the `catalog::scan` STD-003 sites — outside this slice's
  surfaces; `IMP-443` carries the census.

### Note on the design run

`design.md` is edited **out of band**: the run is locked at rev 63 with a
materialised watermark, so its section fingerprints now diverge from the run. That
is the reconcile write surface working as designed — the locked run is why eight
falsified claims had to accumulate through execution instead of being fixed in
flight, and `RV-363`'s standing-risks section records the seam as still open.

Reconcile pass complete — handoff to `/close`.
