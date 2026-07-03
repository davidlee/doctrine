# Review RV-238 — reconciliation of SL-192

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface.** SL-192 solo fork `w/SL-192-p01` (base `4c43b4d0`; commits
`e8470d64` P01 feat, `fc4eee72` lifecycle, `23445fb7` P01 notes, `fe6a68b9` P02
feat, `bfafa056` P02 notes), landed onto `edge` via `--no-ff` merge `136094a7`.
NOT a dispatched slice — no candidate branch; the fork working tree is the
evidence. Audit run from the parent tree (`edge`) per IMP-024 after landing.

**Land-first note.** Solo-fork audit has no candidate-branch analog (unlike
dispatch). The review baton requires the parent tree (IMP-024 refuses `review
raise`/`dispose` on a fork), and the parent tree lacked both the code and the
completed-phase runtime state (`.doctrine/state` is fork-local/gitignored). So
the fork was landed to `edge` first (ledger-sanctioned "merge the fork first"),
phase state reconstructed on `edge` (2/2, → `audit`), then the audit driven here.
Friction recorded to RFC-011 case-notes + ISS captured for the solo-fork gap.

**Lines of attack.**
1. **Conformance algebra** — `slice conformance SL-192` (edge): every touched
   path in a `design-target` selector; any undeclared/undelivered?
2. **VT/VA delivery** — PHASE-01 VT-1/2/3 + VA-1, PHASE-02 VT-1/2/3 present,
   green, asserting membership / conjunction-intersection / root-wise specificity
   (incl. the accepted alpha-earlier-factor boundary) / sidecar presence-through-
   serde / e2e multi-key compose + explain.
3. **Behaviour-preservation (VA-1)** — delivered suites green after the
   `Option<String>`→`BTreeSet` migration; ONLY intended output delta = the
   `explain`/`Spec` byte-form, by intent (no silent golden rewrite).
4. **Design/spec fidelity** — encoding matches SPEC-023 D2/D3/D4 (set-valued
   axes, conjunctive selector, root-wise `(root,depth)` multiset NOT collapsed,
   context-free specificity); locked algebra not re-opened.
5. **Metadata & governance edges** — slice relations correct; declared non-goals
   / tracked follow-ups (design §7) actually captured; REV-019 landing-adjacency.
6. **Gate** — exit 0.

**Gate evidence.** Fork gate: `doctrine check gate` exit 0 (suite + clippy +
fmt). Edge gate: SL-192 code passes; the edge tree carries an UNRELATED
pre-existing blocker (`.doctrine/dispatch/` unclassified gitignore glob, commit
`61eae2ce`, dispatch-domain) captured as ISS-207 — not an SL-192 defect (SL-192
touches neither `.gitignore` nor `src/worktree/`). SL-192 behaviour/conformance
proven by the fork gate + edge conformance 4/4.

**Invariants held.** Specificity context-free (selector only); empty
`Selector.model` ≡ prior `None` don't-care; singleton context/selector byte-
identical to delivered SL-186; total precedence order preserved.

## Synthesis

SL-192 delivers the SPEC-023 conformance fix cleanly. The prompt-cascade engine
(SL-186) now matches SPEC-023 forward-intent: a set-valued model axis
(`BTreeSet<String>` both sides, empty = prior `None` don't-care), two composing
match modes (membership on the context side, conjunction/intersection on the
selector side), and root-wise `(root,depth)`-multiset specificity compared by
derived `Vec: Ord`. The shell surface followed — repeatable `--model` and a
presence-preserving `Sidecar.model: Option<Vec<String>>` (the load-bearing Option
that keeps omitted≠empty through serde). Both phases green.

**Evidence.** Fork gate `doctrine check gate` exit 0 (suite + clippy + fmt).
Edge conformance 4/4 conformant, 0 undeclared / 0 undelivered. All criteria
present and green: PHASE-01 VT-1 (`model_pattern_matches`), VT-2
(`model_conjunction_matches_intersection_misses_proper_subset`), VT-3
(specificity table incl. the accepted boundary), VA-1 (behaviour-preservation —
only the `explain`/`Spec` byte-form changed, by intent, no silent golden
rewrite); PHASE-02 VT-1/2/3 (repeatable `--model` compose, sidecar presence
through `toml::from_str`, e2e explain trace). Design locked via internal + codex
passes; the locked SPEC-023 D2/D3/D4 algebra was not re-opened.

**Standing items, consciously accepted:**
- **Specificity boundary (design §4 INV).** A two-root intersection can sort
  BELOW a one-root alpha-earlier factor — D3's mandated lexicographic
  `(root,depth)` order, encoded and asserted (VT-3), not a defect.
- **FR-007 summary wording (F-3).** The SPEC-023 summary reads unqualified vs the
  precise D3 body; the implementation is conformant to the mechanism. Upstream
  spec-clarity nit, briefed optional.
- **REV-019 / SL-193 `install.rs` adjacency (F-4).** No collision at this land
  (verified empty); a future concern when SL-193's exposed-slot code lands.
- **Process friction (recorded, not SL-192 defects).** Solo-fork audit has no
  candidate-branch analog: the review baton requires the parent tree (IMP-024)
  but the parent tree lacked the code + fork-local runtime state, forcing a
  land-first + phase-state reconstruction. And `review new` succeeds on a fork
  while `review raise` refuses it (IMP-024) — a mint-then-strand trap. Both in
  RFC-011 case-notes. The edge gate is separately blocked by ISS-207
  (`.doctrine/dispatch/` over-broad gitignore, dispatch-domain), unrelated.

No blocker. Slice is conformant and ready to reconcile.

## Reconciliation Brief

### Per-slice (direct edit)
- **slice-192.toml relation (F-1)** — remove the spurious `governed_by ADR-011`
  edge (ADR-011 governs orchestrator-spawn, not prompt-cascade). Load-bearing
  change is the verb: `doctrine unlink SL-192 governed_by ADR-011`. No relink
  target (no prompt-cascade ADR); `references→SPEC-023` + `related→SL-186` already
  carry the real governance.

### Governance/spec (REV) — optional, minor
- **SPEC-023 D3 summary / FR-007 = REQ-328 (F-3)** — OPTIONAL clarify: add the
  shared-prefix qualifier to the "intersections outrank their factors" summary
  (spec-023.md:250 + the FR-007 one-liner) so it matches the precise D3 body
  (spec-023.md:121-123). Minor; REQ-328 is still `pending` and SL-192 is
  conformant to the mechanism regardless. Reconcile may defer to SPEC-023's own
  lifecycle rather than mint a REV.

### Follow-up work captured (provenance only — NOT reconcile write surfaces)
- **IMP-239 (F-2)** — onboard `--model` copy understates the repeatable trait-set
  contract (SL-187 delivery surface). Fenced non-goal, now tracked.
- **ISS-207** — edge gate blocker: `.doctrine/dispatch/` over-broad gitignore
  shadows the committed dispatch ledger + fails the classification parity test
  (commit `61eae2ce`, dispatch-domain). Pre-existing on edge; unrelated to SL-192.

## Reconciliation Outcome

### Direct edits applied
- **slice-192.toml relation (RV-238 F-1)** — removed the spurious `governed_by
  ADR-011` edge via `doctrine unlink SL-192 governed_by ADR-011`. Verified:
  relationships now carry only `references(implements): SPEC-023` — the real
  governance. No relink (no prompt-cascade ADR).

### Governance/spec (REV) — deferred, not authored
- **SPEC-023 D3 summary / FR-007 = REQ-328 (RV-238 F-3)** — deferred to SPEC-023's
  own lifecycle; **no REV minted**. The finding is an OPTIONAL upstream spec-clarity
  nit (add shared-prefix qualifier to the "intersections outrank their factors"
  summary at spec-023.md:250 + the FR-007 one-liner). REQ-328 is still `pending`;
  minting a one-qualifier REV would duplicate SPEC-023's own pending work. SL-192 is
  conformant to the locked D3 mechanism regardless (VT-3 asserts the boundary). The
  clarify lands when SPEC-023 next advances REQ-328. Brief explicitly sanctioned
  this deferral.

### Follow-up / tolerated
- **RV-238 F-2** (follow-up) — captured as **IMP-239** (onboard `--model` copy
  understates the repeatable trait-set contract; SL-187 delivery surface). Provenance
  only; not a reconcile write.
- **RV-238 F-4** (aligned) — REV-019 / SL-193 `install.rs` adjacency: no collision at
  this land (verified empty). No action.
- **ISS-207** — edge-gate blocker (`.doctrine/dispatch/` gitignore, dispatch-domain).
  Pre-existing, unrelated to SL-192. Provenance only.

Reconcile pass complete — every brief item resolved (F-1 applied, F-3 deferred with
rationale, F-2/F-4/ISS-207 provenance). No half-applied REV. Handoff to /close.
