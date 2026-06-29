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
