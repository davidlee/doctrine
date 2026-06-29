# Review RV-192 — reconciliation of SL-176

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Facet:** reconciliation. **Mode:** conformance (post-implementation audit of
a dispatched slice). **Self-audit** (reviewer ≡ author, `--as` role assertion).

**Surface reviewed.** Dispatched slice (pi arm). Evidence refs are immutable:
`review/176` (`b9df004a`, impl bundle), code tip `dispatch/176` (`8a9201cd`).
Audit reviews the **candidate interaction branch** `cand-176-review-001`
(`candidate/176/review-001` @ `910959d8`, base `refs/heads/main`). Hard evidence
(verify-vt, check gate, census) was run from the live coordination tree
`.dispatch/SL-176` (the only tree holding SL-176 code) using **that tree's own
`./target/debug/doctrine` 0.9.0** — the PATH binary `~/.cargo/bin/doctrine` is
stale (0.8.1, pre-SL-176) and silently drops the new `fulfils`/`originates_from`
labels (see Synthesis: census trap). Review verbs were driven from the primary
edge tree (they refuse worktree-resolved roots, ledger §6).

**Lines of attack.**
1. **Migration faithfulness** — did the 124 corpus rewrites land every
   `slices`/`scoped_from`/`drift`-entity edge at its VH-1-approved TO shape, with
   no dangling/illegal edges? (oracle P04 VT-1 + census end-state)
2. **Label retirement** — is `slices` fully retired and `scoped_from` renamed,
   with `fulfils`+degree and `references(originates_from)` legal and rendering?
3. **Conformance algebra** — disposition every undeclared (124) / undelivered (4)
   cell: deliverable, ripple, or scope creep?
4. **Behaviour-preservation gate** — entity-engine suites green unchanged.
5. **Scope fidelity** — does observed match design scope/Non-Goals, and is every
   deviation a recorded, human-approved decision (not silent drift)?
6. **Governance sequencing** — confirm ratification is correctly deferred to
   reconcile, not silently dropped.

## Synthesis

**Closure story.** SL-176 finished Axis B: retired the `slices` label, renamed
`scoped_from`→`references(originates_from)`, introduced the `fulfils` label with a
`{full, partial}` degree facet, added the value-burndown priority post-pass, and
migrated the live corpus. The implementation is sound and the deliverable is
proven, not asserted:

- **VT evidence (coord tree):** verify-vt 19/19 PASS, exit 0. `check gate` exit 0,
  full suite green, zero warnings — the behaviour-preservation gate (entity-engine
  suites unchanged) holds.
- **Migration faithfulness:** the P04 VT-1 oracle (`8a9201cd`) parses the
  VH-1-approved `migration-dispositions.toml` and asserts every relation-layer TO
  edge (103 rows, classes 1–4) exists in the live corpus at its recorded
  (label, role, degree); class-aware multiset oracle (VT-2) over the record. Green.
- **End-state (census, coord 0.9.0):** `label="slices"` 0 (retired), `fulfils` 41
  / `references(originates_from)` 62 — all resolved, zero unresolved/dangling;
  `scoped_from` 0 (renamed). Edge-level corpus grep corroborates.

**Conformance algebra — all five leads disposition `aligned`:**
- *124 undeclared* (F-1) — the corpus relabel deliverable; footprint anticipated
  in design prose, never encoded as a selector. Oracle-proven faithful.
- *supersede.rs undeclared* (F-2) — the lone change is the `append_edge` degree
  parameter ripple; behaviour-preserving, under-declared selector.
- *4 undelivered* (F-3) — benign over-declaration from the design's explicitly
  "coarse" affected-surface; the targets carry only free-text drift (a Non-Goal)
  or no relation row at all. No edit was correct.
- *2 bare-entity drift remain* (F-4) — ISS-041→RFC-003, ISS-048→IMP-148: fit
  neither migration outlet, consciously left under OQ-2 (Class 6), VH-1 approved.
- *governance deferred* (F-5) — correct sequencing per Non-Goal; reconcile's job.

**The census trap (durable gotcha).** Running `doctrine relation census` from the
coord tree via the **PATH binary** (0.8.1, pre-SL-176) reported `fulfils` and
`references(originates_from)` as *absent* — a false negative: the stale binary
drops labels it does not know. The coord tree's own freshly-built
`./target/debug/doctrine` (0.9.0) showed them correctly. Any post-migration corpus
inspection MUST use the build target, never the installed PATH binary. (Compounds
the known verify-vt evidence-tree caveat: edge tree lacks the code; coord tree has
it but only the built binary understands it.)

**Standing risks / tradeoffs consciously accepted.**
- *Selector coverage vs corpus deliverable* — conformance can never be mechanically
  "clean" for a corpus-wide migration unless a `.doctrine/**/*.toml` design-target
  is declared up front. Tolerated here (faithfulness proven by oracle); a process
  note for future migration slices, not a defect.
- *Bare-entity drift residue* — 2 "concerns"-shaped drift edges deliberately
  outlive this slice (no grammar outlet); their decision lives in the disposition
  record, not yet in design prose (optional polish, F-4).
- *S1 regression fingerprint not captured this session* — the full `check gate`
  suite green on the coord tree stands in; baseline-diff needs same-env capture.

**Verdict:** implementation conforms to design and governance; no blocker, no
unreconciled drift. The substantive remaining work is the design-sanctioned
governance authoring, handed to /reconcile below.

## Reconciliation Brief

### Per-slice (direct edit)
- *(optional polish, F-4)* `design.md` migration section — add a one-line xref to
  the OQ-2 bare-entity-drift carve-out (ISS-041→RFC-003, ISS-048→IMP-148 left on
  `drift`), so design canon points at the disposition record. Not required:
  authoritative truth already lives in `migration-dispositions.{md,toml}`.

### Governance/spec (REV) — F-5, the substantive reconcile work
- **Ratifying ADR** (REV new, or amend ADR-016/ADR-010) — ratify the Axis-B
  vocabulary: `references(originates_from)` role, `fulfils` label, `{full, partial}`
  degree facet, and the value-burndown priority post-pass. RFC-003 § "Finish Axis B"
  is the decision-of-record; the ADR is its canon.
- **SPEC-018** (REV) — author/extend the relation-vocabulary spec for spec
  coherence (the durable vocabulary registry).
- **`relation-vocabulary.md`** — ship/refresh the shipped reference doc for the
  consolidated role + label set.
- **RFC-003** — close via REV (its Finish-Axis-B decision is now discharged).
- **Backlog closures** — discharge IMP-207 (19-row retcon rode this slice) and
  IMP-149 (`slices` ambiguity dissolved); confirm IMP-210 / IMP-156 remain open
  follow-ups (close-cascade hint consumer; create-time `--originates-from` flag).

## Reconciliation Outcome

Reconcile pass complete (2026-06-29). All five findings were terminal at entry
(F-1..F-4 `aligned`, F-5 `reconcile`); the substantive write was F-5's deferred
governance. F-1..F-4 were `aligned` conformance dispositions needing no write
(F-4's optional polish applied below).

### Governance / spec authored

- **ADR-018** (`finish-axis-b-fulfils-degree-burndown`) — authored + `accepted`.
  Ratifies RFC-003 § "Finish Axis B"; composes with ADR-016/010/004 and **partially
  reverses ADR-016 §2** (completion is relational via the `fulfils` degree facet).
  `related` → ADR-016/010/004; `SL-176 governed_by ADR-018` linked. Authored directly
  (the REV change grammar has no create-ADR action; ADR-016 precedent). Covers RV-192 F-5.
- **REV-016** (`reconcile-sl-176`, `originates_from` RFC-003) — `done`. One `modify`
  row against **SPEC-018**, hand-landed: `scoped_from`→`originates_from` (renamed +
  widened), new `fulfils` label + `Degree {full, partial}` facet, plus a "Finishing
  Axis B" subsection; companion **relation-vocabulary.md** refreshed (retired `Slices`
  row, renamed role, added `Fulfils` + degree). Rationale in revision-016.md. Covers F-5.
- **RFC-003** → `resolved` (`rfc status`) — Finish-Axis-B decision discharged by ADR-018.

### Direct edits applied

- **design.md** migration § — added the OQ-2 bare-entity-drift carve-out xref
  (ISS-041→RFC-003, ISS-048→IMP-148 left on `drift`, Class 6, authoritative truth in
  `migration-dispositions.{md,toml}`). Covers RV-192 F-4 (optional polish).

### Backlog discharged

- **IMP-207** (spawned_from retcon) → `resolved · done`; **IMP-149** (`slices`
  ambiguity) → `resolved · done`. **IMP-210** (close-cascade hint) and **IMP-156**
  (`--spawn-from` flag) confirmed open follow-ups — not discharged (out of SL-176 scope).

### Withdrawn / tolerated

- None. All findings verified terminal; no withdrawals or tolerations.

Reconcile pass complete — handoff to **/close** (remove coord tree `.dispatch/SL-176`,
`git fetch . edge:main`, `dispatch sync --integrate --trunk refs/heads/main`).
