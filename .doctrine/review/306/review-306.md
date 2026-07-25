# Review RV-306 — reconciliation of SL-229

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** The parent tree (`edge`, `70cf9ea5`). SL-229 was driven
**solo**, not dispatched — `dispatch status --slice 229` reports no coordination
branch, so there is no candidate interaction branch and no evidence-ref caveat.
Reviewed artefacts: the three phase commits (`73b6c29d`, `14a9f9f8`,
`64fcc7ad`), `design.md`, `plan.toml`, `notes.md`, and the published asset
library from the freshly-built in-tree binary.

**Lines of attack.**

1. **Does the mechanism reach a consumer?** The slice's value is not the verb —
   it is agents *running* the round before design. That requires the four
   consumption hooks to be visible in a harness. Probe the distribution chain
   end-to-end (master → embed → release tag → marketplace ref → plugin cache),
   not just the authored master. PHASE-03's `notes.md` explicitly deferred this
   residual here.
2. **Design ↔ implementation conformance.** `design.md` D1–D6 and § CLI verb +
   engine against `src/research.rs` / `src/slice.rs` / `src/commands/guard.rs`;
   ADR-001 leaf tier; ADR-003 advisory-never-a-gate (exit 0 on every path);
   ADR-005 restate line in the skill body; POL-002 (no project-local content in
   shipped text); STD-001 named constants.
3. **Path conformance.** `slice conformance` algebra — every `undeclared` row
   adjudicated as genuine scope creep, a missed design-target declaration, or
   registry noise. Necessary, not sufficient.
4. **Closure-evidence honesty.** Design § Verification alignment claims VT/VA/VH
   levels. Do the VH and the carried risks (A1/A2/R1) actually have evidence, or
   is the slice asserting what it has not yet observed?
5. **Storage-rule and tier claims.** D1 asserts gitignored-in-place, per-worktree,
   harvest-at-close. Verify each leg has a real mechanism, not just prose.

## Synthesis

**The closure story.** SL-229 built what it designed. The engine, the verb, the
skill and the four hooks all exist, are correct, and are gate-green — and the
slice's behavioural product nonetheless reaches nobody yet. That gap is the
whole story of this audit, and it is a distribution fact, not a defect in the
work.

*What was verified clean.* `doctrine check gate` exits 0 — zero clippy warnings,
zero test failures across the workspace. All ten VTs pass (`slice verify-vt
229`). Path conformance, once adjudicated, is the cleanest possible shape: all
twelve `design-target` selectors delivered, zero undelivered, and exactly one
genuine undeclared row (F-3). `src/research.rs` implements design § CLI verb +
engine faithfully and improves on it in one place — the mint/check branch keys
off `baseline.toml`'s existence rather than the directory's, so a research dir
that exists without a baseline mints instead of erroring, which is what EX-1
actually wanted. ADR-001 leaf tier is declared and honest (imports only
`contentset`/`kinds`/`fsutil`); ADR-003's advisory contract holds on every path
(`run_research` returns `Ok(())` in all three branches); ADR-005's restate line
is respected in the skill body and, per D-c, more strictly than design.md's own
sketch — the `/phase-plan` hook says "re-stamp the baseline" in prose rather
than naming `--restamp`, because shipped skill text may not carry flag syntax.
STD-001 constants are named. POL-002 holds: no project-local reference survives
in shipped text. EX-2's `dead_code` demand was met by genuine removal —
`contentset::is_stale_against` had no production consumer and the boolean
relocated to `SetDrift::is_empty`, which `run_research` uses. That is the right
resolution of a "consume it or remove it" instruction, not the easy one.

*The one thing that matters.* PHASE-01 and PHASE-02 are ancestors of tag
`v0.31.0`; PHASE-03 is not, and `64fcc7ad` is not on `origin/main`, which
`marketplace.json` sources. Grepping the live cache this audit's own session
loads its skills from: `research/SKILL.md` is present, and the four hooked
skills contain zero matches for the hook text. So `/research` is installed
everywhere and nothing points at it. The slice's premise — that agents cite a
research artefact instead of recall — depends entirely on those pointers, so
until CHR-048 lands, SL-229 has shipped a capability with no callers. The
authored masters are correct; nothing needs rewriting. The operator was offered
both routes (hold the close for a host-side release, or close with the release
tracked) and elected the latter, which matches how this repo already batches
skill releases. `just release-check` runs a hermetic nix flake build and nix is
absent from the jail, so the audit could not have cut it regardless.

*Standing risks, carried consciously.*

- **R1 — advisory hooks may under-deliver without enforcement.** Unchanged from
  design, and now sharper: it is not merely open, it is *untestable* until
  CHR-048 lands, because the RFC-011 eval that was to judge whether harder
  gating needs an ADR cannot run while no agent sees a hook. Escalation to
  gating still requires an ADR, not a skill edit (D6).
- **The VH is unevidenced (F-2).** "One further real slice driven through the
  round" has not happened; SL-228 was designed and planned after the hooks
  landed and did not use them. What the slice *does* have is the dogfood round
  on itself — `research/research.md` with its `raw/` thread output, cited
  throughout design.md, and the round that surfaced the pre-existing SL-055
  storage convention and thereby overturned the scope's original
  state-tier+symlink wording into D1. That is real evidence that the round
  produces value; it is simply not the independent datapoint the VH asks for.
  Deferred to CHR-048 step 4, not waived.
- **A1** (prose runner deferral suffices for both arms) and **A2** (the fixed
  scope/design/plan baseline path set is enough for v1) remain carried in
  design.md § Open questions. A1 is now marginally better supported: F-6's fix
  filled this repo's own runner socket, so the pi arm's half is concrete.

*Tradeoffs consciously accepted.*

- **Gitignored-in-place storage (D1)** buys per-worktree isolation and zero new
  machinery, and costs an artefact that is invisible to `git status` and
  evaporates silently at close. F-5 found that D1's mitigating claim — "harvest
  at close is explicit" — has no close-side surface; IMP-314 owns it. Accepted
  because the blast radius is bounded: research.md's conclusions are cited into
  design.md by the `/design` hook's own discipline, so an unharvested slice
  loses the audit trail and raw thread output, not the design rationale.
- **Advisory, never a gate (D6).** Deliberate, per ADR-003, and the reason R1
  exists at all. Not revisited here.
- **Conformance noise tolerated as someone else's problem (F-4).** 18 of 19
  undeclared rows were foreign commits swept in by shared-`edge` boundary
  ranges. Both root causes are already owned (IMP-175, IMP-292 #1) and no third
  duplicate was minted — but note SL-229 is the first *solo* datapoint for a
  defect previously evidenced only on dispatched slices, which is worth more
  than the row count suggests.

*Beyond the slice.* Two findings were pre-existing conditions SL-229 merely
walked into: F-7 (five of nine projected reference docs under `.doctrine/` are
stale and, post-SL-227, unrefreshable — IMP-315) and F-4. Both are SL-227
fallout of the same family: minimal projection removed routes without removing
the artefacts or the instructions that point at them. The unifying lesson, now
recorded in `mem.pattern.distribution.skills-source-vs-installed`, is that
authored-and-committed no longer implies reachable, and the only honest check
is to grep the consumer surface.

## Reconciliation Brief

Two items, both per-slice. **No governance or spec surface needs a REV** — no
ADR, policy, standard, spec or requirement was found wrong by this audit, and
every out-of-scope finding was routed to the backlog rather than to canon.

### Per-slice (direct edit)

- **F-3 — `slice-229.toml` selector registry (load-bearing).** Declare the one
  genuine undeclared touch:

  ```
  doctrine slice selector add 229 .doctrine/adr/001/layering.toml \
    --intent design-target \
    --note "ADR-001 layering entry — mandatory for any new src/ module"
  ```

  The registry is what `slice conformance` reads. Do this **first**: a prose-only
  fix leaves the row red. Verify with `doctrine slice conformance 229` — the
  genuine undeclared row should clear, leaving only the foreign-commit noise F-4
  routed to IMP-175 / IMP-292.

- **F-3 — `design.md` § Code impact summary (human mirror).** Add a row:
  `| .doctrine/adr/001/layering.toml | leaf-tier registration for the new module |`.
  Secondary to the registry write, not a substitute for it. Worth carrying the
  general lesson in the same edit if it reads naturally: a new `src/` module
  **always** implies an ADR-001 `layering.toml` entry, because the
  `architecture_layering` gate fails `Unclassified` without one — so any future
  design declaring a new module should declare that path up front.

- **F-2 — `design.md` § Verification alignment (honesty edit).** The VH currently
  reads as satisfied-by-intent. Annotate it to state what is actually true at
  close: the dogfood round on SL-229 itself is the pre-design evidence and is
  real, but the "one further real slice driven through the round" datapoint is
  **deferred to CHR-048 step 4**, because the hooks are not yet distributed. Same
  edit should note that R1 is consequently untestable until CHR-048 lands. Do not
  quietly downgrade the VH — record the deferral and its cause.

### Governance/spec (REV)

None.

### Explicitly off-surface

- **`plan.toml` is not touched.** No criterion changes; `PHASE-NN` and
  `EN-/EX-/VT-` ids are immutable-append. PHASE-03's EX-2 is satisfied by
  interpretation (`notes.md` D-a: the contract is "harness-visible copy matches
  master", since the criteria's named paths — `.agents/skills/`,
  `.doctrine/skills/` — were killed by SL-227). That interpretation is recorded
  in `notes.md` and this ledger; it is not a plan edit and must not become one.
- **Backlog items are already minted and need no reconcile write**: CHR-048
  (F-1, F-2 — release the hooks), IMP-314 (F-5 — harvest pointer at close),
  IMP-315 (F-7 — stale projected reference docs). F-4 routed to existing IMP-175
  and IMP-292. F-6 was fixed during the audit (`.doctrine/governance.md`
  § Research agents + boot regen).

## Reconciliation Outcome

### Direct edits applied

- **`slice-229.toml` selector registry (RV-306 F-3, load-bearing).**
  `doctrine slice selector add 229 .doctrine/adr/001/layering.toml --intent
  design-target` — declared the one genuine undeclared touch. `slice conformance
  229` moved from 19 undeclared / 0 undelivered / 12 conformant to **18
  undeclared / 0 undelivered / 13 conformant**; the residual 18 are the
  foreign-commit rows F-4 routed to IMP-175 / IMP-292, not SL-229's.
- **`design.md` § Code impact summary (F-3, human mirror).** Added the
  `.doctrine/adr/001/layering.toml` row, plus a note carrying the general
  lesson: a new `src/` module always implies an ADR-001 `layering.toml` entry,
  because the `architecture_layering` gate fails `Unclassified` without one.
  Secondary to the registry write, which is what `slice conformance` reads.
- **`design.md` § Verification alignment (F-2, honesty edit).** The VH now reads
  *partially satisfied at close* — the dogfood round on SL-229 itself is stated
  as the real pre-design evidence (it overturned the scope's state-tier+symlink
  wording into D1), and the "one further real slice driven through the round"
  datapoint is recorded as **deferred to CHR-048 step 4, not waived**, with its
  cause: the hooks are authored but undistributed (F-1). The same edit records
  that R1 is consequently untestable until CHR-048 lands.
- **`design.md` § Open questions / risks (F-2).** R1 annotated with the same
  close-time fact and cross-referenced to § Verification alignment.

### REVs completed

None. The brief carried no governance/spec items — no ADR, policy, standard,
spec or requirement was found wrong by this audit, and every out-of-scope
finding was routed to the backlog rather than to canon.

### Withdrawn / tolerated

None — all seven findings are `verified`. Findings whose remediation lands
outside this reconcile pass are carried by already-minted backlog items, not by
disposition changes:

- **F-1, F-2 (second leg)** → CHR-048 — release the hooks; step 4 carries the
  deferred VH datapoint.
- **F-4** → existing IMP-175 / IMP-292 #1 (shared-`edge` boundary-range noise).
  No third duplicate minted.
- **F-5** → IMP-314 — close-side harvest pointer for gitignored research/.
- **F-7** → IMP-315 — stale projected reference docs, post-SL-227.
- **F-6** was fixed during the audit itself (`.doctrine/governance.md`
  § Research agents + boot regen).

### Off-surface, confirmed untouched

`plan.toml` — no criterion edited; `PHASE-NN` and `EN-/EX-/VT-` ids are
immutable-append. PHASE-03's EX-2 remains satisfied by the interpretation
recorded in `notes.md` D-a, which stays a ledger fact and did not become a plan
edit.

Reconcile pass complete — handoff to `/close`.
