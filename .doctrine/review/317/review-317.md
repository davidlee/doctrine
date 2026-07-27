# Review RV-317 — code-review of SL-231

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** The landed SL-231 PHASE-01..03 source delta on `dispatch/231`
@ `0fe9572b` (source-identical to coord HEAD `24edc3776`; the extra commit is
funnel-only). ~6,800 added lines across `src/observation/**`, `src/fsutil.rs`,
`src/commands/observation.rs`, `src/commands/guard.rs`, `src/entity.rs`,
`tests/e2e_observation.rs`, `tests/architecture_layering.rs`.

**Why now.** User-interposed between PHASE-03 reap and PHASE-04 spawn. PHASE-03
required three orchestrator cleanup turns for defects that passed every gate,
all found by ad-hoc diff-reading rather than systematic review — so the coverage
question is open by construction. PHASE-04 (MCP adapter) and PHASE-05 (dogfood
activation) both build on this code, and PHASE-04 makes `Service` its second
consumer.

**Reviewer selection.** pi/deepseek read-only pass, per
`mem.fact.dispatch.deepseek-review-capability`: the observed failure mode on
this slice was *self*-review, not review capability — the same model wrote three
gate-invisible defects and self-reported success each time, then fixed all three
correctly first try once diagnosed. A separate turn reviewing another turn's
output is precisely the gap this closes. Every finding is adjudicated against
the code by the orchestrator before disposition; a bad turn is a bad turn, not a
capability verdict.

**Lines of attack** (prior-probability, from where defects already clustered —
not a findings list):

1. **Adapter/service layering** — `src/commands/observation.rs` is the biggest
   surface and hosted or caused all three PHASE-03 defects. Does the adapter
   still carry logic that `Service` / `query` owns? Defect 2 was exactly this
   (a hand-rolled `filter_and_order` duplicating `query::query`).
2. **`escape_hostile`** — twice-fixed, security-relevant, and its bug class
   (byte-vs-char) is easy to reintroduce at a new call site. Audit EVERY call
   site's `EscapeContext` choice, not just the function body.
3. **`store.rs` concurrency claims vs design §3.2** — the crash-safety wording
   is deliberately narrow (no protection from a malicious local actor swapping
   directory components). Confirm neither code nor comments make a stronger
   claim than the design licenses.
4. **Test INPUT coverage, not test presence** — the UTF-8 corruption bug
   survived a green suite purely because no test string contained a non-ASCII
   byte. Which input classes are unexercised? (non-ASCII, C1 controls, NUL,
   limit boundaries, RTL/bidi, combining marks, lone surrogates in JSON.)
5. **`Service` API surface fitness** — PHASE-04's MCP adapter is its second
   consumer. A contract shaped only around the CLI's needs will distort it.
6. **ADR-001 layering + POL-002** — `observation = "leaf"`; no clock, RNG, git,
   disk, env, terminal, or MCP types in `wire` / `resolve` / `query`.
7. **STD-001** — no magic strings; the PHASE-02 cleanup already caught a
   duplicated publication-temp prefix constant.

**Out of scope.** PHASE-04/05 `verify-vt` rows (UNATTRIBUTABLE / FAIL) are
unimplemented-phase artifacts, not defects. ISS-263 and ISS-264 are filed funnel
defects, not findings. Pathologies outside the landed delta go to `backlog new`,
not this ledger.

## Synthesis

**Overall: revision-required.**

### Synopsis

The observation ledger is, structurally, good work. The module split the design
specified is the module split that got built: `wire` / `resolve` / `query` are
genuinely pure — a grep with a positive control finds no clock, RNG, disk, or
environment access anywhere in them, and `store` really is the only disk seam.
ADR-001 leaf classification holds and `tests/architecture_layering.rs` gates it
mechanically rather than decoratively. STD-001 is well served: every limit,
schema discriminator, path component, and the publication temp prefix is a named
constant referenced from both definition and use. The publication primitive is
the strongest single piece of the delta — bytes written and closed before a
no-clobber `hard_link`, the destination never opened for write, temps cleaned on
every path but one — and, unusually, its doc comment mirrors design §3.2's
deliberately narrow crash-safety wording *exactly*, including the explicit
refusal to claim protection against a malicious local actor. It was asked to
find over-claim in that module and there is none.

The defects cluster where the handover predicted they would, and they share one
ancestry. Three of the four most serious findings are the same two bug classes
that PHASE-03's cleanup turns already caught once each, recurring in locations
the cleanup did not reach:

- **Byte-vs-char reasoning over `str`.** The Latin-1 escaper was fixed; the
  identical mistake survives in `shard_dir`, which byte-slices a UTF-8 uid and
  **panics** on three of the six public verbs (F-2, reproduced against the built
  binary). It survived a green suite for exactly the reason the escaper bug did:
  no test string on that path carries a non-ASCII byte.
- **Parallel implementation.** `filter_and_order` was deleted from the adapter;
  the same shape survives as dead `Service::load_all_resolved` re-implementing
  `query(Projection::History)` with a byte-identical comparator (F-7), and
  `run_list`/`run_search` remain 43 duplicated lines of the very pipeline that
  cleanup consolidated (F-6).
- **Incomplete application of a correct fix.** EX-5's row-injection repair gave
  one escaper an `EscapeContext` and applied it to payload summary and detail —
  but envelope *metadata* still renders raw at nine loci, so terminal escape and
  row injection are demonstrably re-opened through `recorded_at`, a field no
  validator constrains and no read path re-checks (F-1, reproduced end to end).

That is the standing risk worth naming: **this slice's cleanups have fixed
instances rather than classes.** Each individual repair was correct and each was
verified; none was swept for siblings. The remedy is not more review but a
sweep-for-siblings step whenever a defect class is identified.

The second theme is quieter and more corrosive: **capability that exists, is
tested, and reaches nobody.** The resolver computes the full per-control
diagnostic set design §4 specifies, and `load_all` produces record diagnostics —
and every CLI call site discards both, with no render path anywhere (F-3). The
unit tests are green over behaviour the product surface does not expose, so a
corpus with an unparseable record reads silently short. For a ledger whose entire
purpose is trustworthy evidence, silent partial reads are the expensive failure.
Adjacent to it, three tests assert a tautology, the opposite of their name, or a
classification instead of the diagnostic they are named for (F-4) — and one of
those hides a genuinely unverified design §6 item, since nothing anywhere creates
a different *kind* at an existing uid.

### Tradeoffs consciously accepted

- **F-5 → IMP-329.** Three hand-enumerations of 33 facet fields are a real drift
  hazard, but they are *consistent today* (verified: empty symmetric difference),
  nothing upcoming depends on the refactor, and collapsing 450 lines of walk into
  one field source deserves its own scope rather than riding a security fix. The
  cheap cardinality guard is explicitly **not** deferred with it.
- **F-11 downgraded to nit**, against the raiser's `major`. Design §3.2 licenses
  a leftover temp before publication and the leaked name is inert by construction
  — reserved prefix, skipped twice by corpus loading. Severity honesty runs
  downward too.
- **F-1 rated blocker and left open.** Every finding is disposed but none
  verified, deliberately: verifying would retire the close-gate teeth before a
  single fix exists. The RV stays active so the blocker gates SL-231's closure
  until the repairs land.

### On the reviewers

The user's correction was right, and the evidence supports it. Three read-only
deepseek passes produced real findings that survived independent adjudication:
the escape-bypass blocker (accurate line numbers, correct threat model, correct
severity), the discarded diagnostics, the temp leak, and the whole VT assertion
audit. The failure mode was **not** capability — it was calibration. Pass A's
line numbers were systematically ~410 off (numbered against a diff hunk, not the
file), so every claim needed re-derivation against the source; and Pass B
declared "no production-code panics on hostile input" **clean** while a reachable
panic sat one module away in the file it had open. Both are reasons to adjudicate
every finding against the code — which is required regardless — not reasons to
stop using the reviewer. The single most serious finding after F-1 came from
orchestrator probing, not from any pass; the two are complementary, and running
them lens-diverse rather than phase-shaped is what surfaced the cross-phase
contract drift a per-phase pass structurally cannot see.

### Haiku

Green suite, ASCII-blind —
the same slice twice, and the fix
that stopped one door short.
