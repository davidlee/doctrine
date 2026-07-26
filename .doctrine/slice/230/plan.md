# Implementation Plan SL-230: Memory body-write seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases: one engine primitive, three CLI phases, one MCP adapter, one
`validate` change. The shape follows the design's two altitudes (§ 5.1) — a
kind-agnostic body writer at engine, policy composition at command — and then
splits command-tier work by *seam*, because `record` and `edit` do not write the
body the same way. `record` rides the existing scaffold fileset; `edit` uses the
new `write_body`. Treating them as one phase would hide that.

The plan is deliberately not four larger phases. Every phase here ends green with
a criterion that can fail, which matters more than usual on this slice: `entity.rs`
and `validate` are shared machinery under R3, so each step has to prove it did not
move existing behaviour before the next one builds on it.

## Sequencing & Rationale

**PHASE-01 before everything.** `write_body` is the product's first prose-write
path and D1 makes it reusable at engine tier on purpose — whatever shape it takes
is inherited by `spec edit`, `backlog edit` and the rest. Landing it alone, with
its own tests and no caller, is what keeps that decision reviewable instead of
retrofitted. It is also the only phase with no dependency, so it costs nothing to
put first.

**PHASE-02 does not depend on PHASE-01** and is sequenced second only for
narrative order. `record`'s body goes through `memory_scaffold`, substituting the
rendered template exactly as `seed_by_key` already does; the transactional
`materialise_named` write is preserved rather than replaced. Two mechanisms, one
per verb, and no third — which is what "no parallel implementation" means here.
If phases are ever run in parallel, 01 and 02 are the disjoint pair.

**PHASE-03 is where the ordering risk lives.** `run_edit` starts writing two
files and stops being atomic across them. The order is not the obvious one: the
first draft put `write_body` first on the reasoning that the TOML *content*
depends on `body_changed`, and RV-307 F-3 showed that reasoning permits
`edit --body - --trust bogus` to rewrite `memory.md` and *then* fail — mutation on
an argument-validation failure. What must follow the body write is the TOML
**write**, not the TOML **computation**. EX-4 is that criterion, and it is the
reason the phase exists as its own step. R1's residual crash window (changed body,
stale `updated`) is accepted; full two-tier atomicity would mean routing `edit`
through the fileset/rollback machinery, which is a refactor of a shared write path
disproportionate to a window this narrow.

**PHASE-04 separates invalidation from body-write** so each has a failing
criterion of its own. After 03, body edits work and do not clear the attestation;
after 04, they do. The discipline that matters is EX-4: the clearing is driven by
*comparison*, never by which flags were supplied, and it must be asserted on
**scope** rather than title. `apply_edit` already compares-before-assigning for
the scalars, so a title-based test passes even with `claim_snapshot` absent — it
proves nothing. Scope is the one field where the comparison and `apply_edit`
genuinely diverge, which is why the assertion goes there. That is RV-307 F-17,
and it was raised against this design's own test matrix.

**PHASE-05 last of the write phases,** so the MCP arm has one finished core to
delegate to. Sequencing it earlier would invite a second policy site — the exact
divergence F-10 was raised about. The `body_mode`-without-`body` rule is one rule
instanced twice, and its two legs are split across VT-1 and VT-2 because the
mandate carries a single `test_file` and the surfaces live in different ones.

**PHASE-06 depends on PHASE-04, not on the body phases.** Under D8 a *verb* edit
clears the stamp, so there is no `verified_sha` left to compare and the staleness
check never runs on that path. The check only has meaning on the **hand-edit
bypass**, which is the whole reason D5 exists — so it must be built after the
clearing exists, or its tests will assert against a path the product no longer
takes. This is A4, a test-gap the internal review caught before any code was
written.

The phase carries the one criterion this plan would most like to keep: EX-4, the
falsifier. D5's pathspec was the item directory until the confirming pass measured
it and found it reports drift on every anchored memory in the corpus — because
`verify` stamps `verified_sha`, the stamp is then committed, and that commit
touches the directory being measured. The check flagged the sanctioned flow. It is
registered as I13 with a test whose only job is to fail if the pathspec ever
widens back.

## Notes

**Where this plan is thin, stated rather than discovered later.**

Body size is unbounded by design (R2): anyone who can run the verb can already
write the file, so a cap is theatre rather than a boundary. What PHASE-03's VT-2
pins is the *read-time* defence — the per-render nonce and `data, never
instruction` framing — because that is where the defence actually lives. There is
no write-time escaping on the `.md` tier to bypass; markdown bodies are stored
verbatim by design, and the draft's contrary claim was corrected by the internal
review (§ 10, A3).

R4 runs **unmitigated** for the life of this slice. D4/D8 mean every claim-field
edit costs a re-verify, and the gate relaxation that made that affordable left
with D3 for SL-232 (DEC-027). This is accepted friction, not incorrectness — the
state it replaces has the same friction *plus* a stamp that survives a claim
change — and it is the reason to sequence SL-232 next rather than something else.
No phase here should try to soften it.

Masters stay uncovered by every invalidation path (R5). They are unanchored and
`collect_all` never scans them, so neither D5's body drift nor D8's field-clearing
reaches a master edit. D6 puts them out of scope; PHASE-06 EX-6 exists to stop a
future reader inferring coverage the slice does not have.

Two things this plan must not grow. It must not specify exclusion sets, claim
surfaces, or pathspec construction — SL-232 owns the gate, and where PHASE-06
needs a git fact the gate also needs, the design states it locally rather than
cross-referencing. And it must not adopt `write_body` for a second kind: OQ-4
keeps that question open on purpose, and D1's whole point is that adoption is
later a caller change rather than a redesign.
