# Review RV-424 — reconciliation of SL-284

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance audit, self-driven (raiser = responder).
**Subject:** SL-284, rung 1 of the target ladder. Audit range
`9b368b541^..38d1f4f9a`; implementation range `9b368b541..2841ac1d3`.
**Yardstick:** `design.md` sec-2..sec-7 (sec-6's clause table and sec-7's
selector/test intent in particular), `plan.toml` PHASE-01..05 `EX`/`VT`,
`slice-284.md` § Scope and § Non-Goals.

### Lines of attack

1. **Skills routing (design sec-2).** Three harnesses carry
   `skills = ".agents/skills"`; `skills_via` must be gone from the struct, its
   both-routes validation, and every live reference. The shared-dir collapse must
   be *unchanged* machinery (`Member.links` as `BTreeSet`), not new code. The
   per-link gitignore claim ("no new code") must hold.
2. **Delegation removal (design sec-3 + the deletion table).** Every row of the
   table gone; `run_forward_steps` ends at `Leg::Convergence`; no orphaned
   helper, constant or import; no live doc/comment teaching npx.
3. **Refusal (inq-7).** An unregistered `--agent` refused *before any write* on
   every path, naming the registry and suggesting `universal`; the interactive
   answer path on the same rule.
4. **Selection (RV-423 F-1).** `--only-memory` subsets to the memory pair through
   one helper for preview *and* live; the two former call sites now agree.
5. **Pi boot adapter (design sec-4).** `Harness::Pi` total over parse / label /
   import / detect; the two arms disjoint on *every* path including error paths;
   non-applicable legs reported honestly rather than silently skipped;
   `McpOutcome` conditionalisation total, so a pi arm can never name a codex or
   claude MCP file; activation notice on write, silent on no-op, never claiming
   activation (POL-003).
6. **Pi extension location (design sec-5).** `PI_EXT_DIR_REL` in the runtime
   tier; the pure `.pi/settings.json` planner total over the eight cases design
   enumerates, preserving foreign entries and user data; the shell arm honest
   about symlink/unreadable/malformed (STD-003); the outcome reaching both
   `refresh_touched` and `refused_wiring`.
7. **Spec revision (design sec-6, plan PHASE-05 VA-2).** REV-074's row set and
   prose enumeration must follow the design's clause table row for row, and both
   required spec searches must be dispositioned.
8. **Conformance algebra.** Every `undeclared` and `undelivered` cell in
   `doctrine slice conformance 284` is dispositioned: registry drift, undeclared
   scope, or a defensible consequence.
9. **Test integrity.** The rewritten suites must assert the *new* behaviour
   (design sec-3's table), not merely have assertions inverted; and a test that
   would still pass with the feature reverted is not evidence.
10. **Undeclared shared-machinery change.** `src/fsutil.rs`'s `write_atomic` now
    preserves an existing target's mode — a global change to a leaf primitive
    that neither `design.md` nor `slice-284.md`'s affected surface declares.

### Invariants held

- The plan is not higher authority than the design; a divergence between them is
  a finding, not a licence to pick one.
- `plan.toml` criteria ids are immutable-append — a divergence requiring a
  criterion *change* is a design/plan escalation, never a direct-edit brief item.
- A degraded read is disclosed (STD-003), never silently skipped.
- One writer per RV: this ledger is written only from this tree.

### Evidence gathered before raising

- `doctrine check gate` → exit 0, green.
- `doctrine slice verify-vt 284` → every VT row PASS (PHASE-05 has none; its
  VA-1/VA-2 are agent-judged).
- `doctrine slice conformance 284` → 10 conformant, 12 undeclared, 1 undelivered.
- `git diff 9b368b541..2841ac1d3` read in full; REV-074 read; both required spec
  searches re-run.

## Synthesis

Twelve findings, all terminal: four `fix-now` (landed in `3b7090881`), three
`design-wrong` (brief below), three `follow-up` (`8557a881f`), two `tolerated`.
No blocker. Gate green before and after the fixes; every VT row PASS.

### The closure story

The slice does what it set out to do, and the interesting part is that the
evidence for that is not the VT table — it is the shape of the residue.

The mechanical work is sound. Three harnesses carry `skills = ".agents/skills"`,
`skills_via` is gone from the struct and from every live reference, the npx
delegation and its runner seam are deleted outright, an unknown `--agent` is
refused before any write on every path, and `--only-memory` now subsets through
one helper that both the preview and the live convergence share. The pi boot
adapter is a real split, not a rename: the arms are disjoint on the error paths
as well as the happy ones, `McpOutcome` makes "not applicable" a type rather than
a comment, and `refresh_touched` / `refused_wiring` carry the settings outcome
into the commit hint and the boot stamp respectively. Most of this is pinned by
tests that would fail if the behaviour were reverted — I checked that claim
rather than accepting it, and the one place where it was not true (the hinted-block
splice test) is `tolerated` with the sibling test named as the real guard.

Two independent read-only reviews of the diff found the four defects worth fixing,
and both were findings the per-phase reviews could not have found: they were
scoped to a phase's own diff, and the residue (a stale leg comment, a partial-write
error path, a fallback message that prescribed an impossible edit) lives in the
seams *between* phases. The handover had already flagged one of them in passing;
the audit's whole-range read is what converted it into a finding with evidence.

### What the four fixes changed

`3b7090881` closes a real honesty gap and three smaller ones:

- The pi arm's settings leg runs last by necessity, so a bare `?` there destroyed
  the three extension outcomes that had already landed. The install printed
  "refresh failed" over three unregistered files on disk. The error now names the
  directory and each outcome (STD-003).
- The settings fallback told operators to "add this line to its extensions array"
  whether or not `extensions` was an array, and the activation notice then promised
  that `/reload` would load files nothing had registered (POL-003). Both now
  describe the actual state.
- Two comments still described the deleted `Leg::Skills`, and a route-less registry
  entry rendered a dangling `" for <agent>"`.

None of these is large. All of them are the class of thing that makes a later
agent distrust a report.

### Standing risks

1. **DEC-377 is unmarked drift** (F-1). The slice's behaviour is governed by the
   newer DEC-388/DEC-389, but DEC-377 still reads as a live instruction to
   delegate unregistered agents to npx and to route pi through codex. A reader who
   lands on it alone will reimplement what was deleted. This is the audit's most
   consequential finding because nothing in the codebase will ever surface it.
2. **`write_atomic` is now a different primitive** (F-3, F-4, IMP-520). Every
   atomic write in the binary preserves the target's mode. The steady-state
   disclosure is fixed; the temp is still created before the mode is applied, and
   special bits are still copied. The window is short and the failure rare, but
   the fix does not fully close the threat it was written for.
3. **The pi settings write is not symlink-safe on the other two arms** (F-11,
   IMP-521). Pi guards it; Claude and codex do not. The asymmetry is now a
   documented follow-up rather than a surprise.
4. **`install_append_system` remains unreachable** (F-10, ISS-527). SL-119 is
   `done` and its delivery path has no caller. Pre-existing and correctly scoped
   out of this slice — but a `done` slice whose behaviour cannot run is a claim the
   corpus should not make.
5. **Conformance will stay red until the registry is corrected** (F-7). This is
   the one finding whose remediation is mechanical and whose absence would quietly
   degrade every later slice's drift signal.

### Tradeoffs consciously accepted

- **Whole-document JSON round-trip** over a textual splice (F-6). Foreign entries
  keep their order and structure; a foreign integer above 2^64 is not preserved
  bit-exactly. The alternative mechanism is materially more complex for a
  vanishingly rare input.
- **No migration code** (DEC-387). The maintainer's checkouts still hold 40 npx
  skill copies under `.agents/skills`, 33 under `.pi/skills`, a tracked
  `skills-lock.json` and the old `.pi/extensions/doctrine/`. That is VH-1 at close,
  by design, not an omission.
- **A weaker splice test than its name claims** (F-12). Tolerated because the
  sibling test holds the line and a literal fixture would drift from the renderer.

### What this audit did not do

`VH-1` (one-off checkout cleanup) and `VH-2` (live pi and codex sessions loading
skills through the symlinks, pi loading extensions from the runtime dir) are
close-time and human-facing; they remain outstanding by design. `REV-074` is
approved and applied at reconcile, not here. No phase sheet was harvested beyond
checking it carried nothing the notes had missed.

## Reconciliation Brief

Every non-`aligned`, non-`tolerated` finding that touches design or governance,
grouped by the surface `/reconcile` is allowed to write. Findings marked
`follow-up` and `fix-now` are **not** here: the fixes landed in `3b7090881`, the
follow-ups live in backlog (`8557a881f`), and neither is a reconcile surface.

### Per-slice (direct edit — `design.md`, `slice-284.md`)

1. **design.md sec-5** — record the shared-primitive change (RV-424 F-3).
   The pi settings write required hardening `fsutil::write_atomic`, which now
   preserves an existing target's mode; a rewrite of a private `0600` settings
   file would otherwise republish it at the umask default. Cite PHASE-04's review
   finding as the origin. Do **not** fold the residual gaps in — they are
   IMP-520.

2. **design.md sec-5** — record the accepted serialisation caveat (RV-424 F-6).
   The planner rewrites the whole settings document, so foreign entries keep
   their order and structure but numeric values outside i64/u64 are not preserved
   bit-exactly. This is the caveat that makes "Other entries are preserved in
   order" read correctly.

3. **design.md sec-3** — the rewritten-test table gains
   `tests/e2e_install_ownership.rs` (plan PHASE-01 EX-4 rewrites its
   `pi_without_npx_*` tests; the design's table names the other four suites
   only) (RV-424 F-7).

4. **design.md sec-6** — add a **note**, not a clause row (RV-424 F-1): DEC-377
   is superseded at reconcile by a successor decision (see the knowledge-record
   item below). A DEC is **not** a legal `revises` target — `src/relation.rs:668`
   restricts `revises` to `[SPEC, PRD, REQ, ADR, POL, STD]` — so DEC-377 must not
   appear in the REV clause table. Record the supersession so a reader of sec-6
   knows the decision half was handled elsewhere.

5. **slice-284.md § Affected surface** — add `src/fsutil.rs`,
   `install/doctrine.toml`, `install/doctrine.toml.example`, `src/dtoml.rs`.
   Four paths the slice touched that the scope document does not name
   (RV-424 F-3, F-7).

### Selector registry (`slice-284.toml`, via `doctrine slice selector`)

6. **The load-bearing change for RV-424 F-7.** `doctrine slice conformance 284`
   reads these rows, so a prose-only fix leaves it red:
   - `doctrine slice selector rm 284 src/commands/cli.rs` — it was conditional
     in design sec-7; the condition resolved false.
   - `doctrine slice selector add 284 README.md --intent design-target` and
     remove the existing `scope-relevant` row — design sec-7 names it a
     design-target, which is why conformance could not match it.
   - `doctrine slice selector add 284 tests/e2e_install_ownership.rs --intent
     design-target`.
   - `doctrine slice selector add` for `install/doctrine.toml`,
     `install/doctrine.toml.example`, `src/dtoml.rs`, `src/fsutil.rs` — or record
     them as accepted consequences in design sec-3/sec-5 and leave them out; pick
     one, do not leave the algebra red without a disposition.
   - `.gitignore` and `.doctrine/revision/074/**` need no selector: install's own
     output, and authored governance rather than a code path.

### Knowledge record (NOT a Revision)

7. **DEC-377 goes through `supersede`, not `REV`** (RV-424 F-1).

   Correction, recorded because the first draft of this brief got it wrong: a
   Revision cannot revise a knowledge record. `src/relation.rs:668` gives
   `revises` the target kinds `[SPEC, PRD, REQ, ADR, POL, STD]`, so
   `doctrine revision change add 74 --target DEC-377` is **refused**. This was
   caught by corpus memory `mem.fact.revision.no-knowledge-record-target` and
   then verified against the source rather than taken on trust.

   The route, in order — every step is separate and skipping one leaves drift:
   1. `doctrine knowledge new decision "<successor title>"` — mint the successor.
   2. Author its `[facet]` with `doctrine knowledge edit decision <ID>`:
      - **global installs converge without pruning** — unchanged from DEC-377,
        this slice does not touch it;
      - an explicit `--agent` absent from the registry is **refused before any
        write**, naming the registered harnesses and suggesting `universal` for
        another harness that reads `.agents/skills`; no npx delegation survives;
      - pi has its own boot adapter (`[harness.pi] boot = "pi"`, DEC-389), so
        codex's arm writes no pi files and pi's arm writes no codex files.
   3. `doctrine supersede <NEW> DEC-377` — writes `supersedes` on the successor
      and `superseded_by` on DEC-377.
   4. **`doctrine knowledge status DEC-377 superseded` — a separate step.**
      `supersede` writes only the relation edges; without this, DEC-377 stays
      `accepted` and `doctrine validate` reports *corpus clean*, so nothing
      surfaces two live records for one rule.
   5. `doctrine link <NEW> shapes SL-284` — DEC-377 carries `shapes: SL-281`, and
      relations do not migrate with a supersession.

   This is the half REV-074 cannot cover. **Both halves are needed** — the DEC
   supersession here and the spec payload in item 8 — because SPEC-009's `D4`,
   SPEC-010's `D2`/`REQ-175` and SPEC-011's `REQ-480` restate the same claims in
   their own words.

### Governance/spec (REV)

8. **Approve and apply REV-074 as raised** (no DEC row; see item 7), then
   `doctrine spec validate` on SPEC-009, SPEC-010 and SPEC-011 (design sec-7's
   gate: "the REV approved and applied"). `REQ-480`'s status row is covered by
   item 9.

9. **REQ-480's `status` row (pending → active) may be applied.** The handover
   made it conditional on the audit confirming the landed pi adapter. It is
   confirmed: `Harness::Pi` with its own `install_refresh` arm; extensions under
   `.doctrine/state/pi/extensions`; `.pi/settings.json` registration with a
   reported outcome; disjoint arms verified by
   `codex_only_boot_install_writes_no_pi_files` and
   `pi_only_boot_install_writes_no_codex_files`; the four PHASE-04 VT rows PASS;
   gate green. Apply it with the rest of REV-074, not before.

### Deliberately not on any reconcile surface

- **`plan.toml`** — no finding required changing a criterion, so the
  immutable-append rule is not engaged. The stale-comment and empty-offer defects
  (F-8, F-9) were code, not criteria.
- **IMP-520, IMP-521, ISS-527** — backlog items, already committed. Reconcile
  should not touch them.
- **VH-1 / VH-2** — close-time, human-facing, outstanding by design.
