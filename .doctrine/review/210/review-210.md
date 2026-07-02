# Review RV-210 — design of SL-187

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

**Aspect under trial:** the *design intent* of SL-187 (`design.md`) — the
delivery-half of the prompt cascade. Not the implementation (none exists), not
the plan (none exists). Design-facet, `--raiser inquisitor`.

**Subject.** SL-187 delivers SL-186's inert `prompt resolve` engine to live
agents at session start, splitting content by cache property: a cache-stable,
model-agnostic boot sector (governance + universal hymns + inlined onboarding
memories) vs a cache-busting `doctrine_onboard` supplement (model band). Touches
live shared surfaces: `boot.md`, `doctrine_onboard`, pi extension, memory tags.

**Lines of interrogation:**

1. **Contract fidelity to SL-186.** Does the consumed `prompt resolve --role
   [--harness --model --arm --stage --band]` signature (§5.2) match SL-186's
   *locked* contract exactly? A drifted signature is heresy — the parallel
   dispatch premise (§3) collapses if the interface is misquoted.
2. **Behaviour-preservation gate (the spine).** §5.2 claims boot's entity-derived
   sections + assembly logic are "untouched", only additive. Is that achievable
   given the footer→inline substitution (§5.2, D3) *removes* the Onboarding
   footer? Removal is not additive. Interrogate whether the golden churn is truly
   "once" and whether dispatch/onboard suites can stay green *unchanged*.
3. **INV-D1 axis-invariance.** Disk `boot.md` must be model-agnostic and identical
   regardless of `--role`/`--harness`. Does anything in the design leak a
   role/harness/model axis onto the disk artifact? The tier-2 `@`-import contract
   depends on it.
4. **INV-D2/D4 cache-hold under concurrency.** The byte-identical-per-turn posture
   (CHR-033) is load-bearing for the token win. Does unconditional unstale on every
   `resolve` (incl. worker spawns) genuinely stay a no-op under stable governance,
   and stale-by-≤1-cycle (never torn) under concurrent writers? Probe the
   `write_if_changed` last-rename-wins claim.
5. **Onboarding-inline correctness (INV-D3).** `collect_all` union (items ∪ shipped,
   local-wins), deterministic key order, no model content ever inlined. Is the
   local-wins collision truly intended + observable, not silent? Is the "small-by-
   construction" assumption load-bearing without a budget (OQ-1, R3)?
6. **Model band honesty (D4/F14).** Is the floor/supplement/no-true-ceiling framing
   sound, or does any correctness invariant secretly rest on the model band?
7. **Unresolved scope & open questions.** OQ-1 (budget) and OQ-2 (single vs combined
   emit) left open — are they safe to defer to plan, or do they hide a design fork?
8. **Mortal sins of `/canon`.** Magic strings (STD-001), platform coupling (POL-002),
   silent error handling, hidden randomness, duplicated concepts, terminology drift.

Doctrine held against: ADR-011 (capability altitude), ADR-002 (orientation class),
CHR-033 (cache posture), POL-002, STD-001, the SL-186 locked contract, the
behaviour-preservation gate.

## Synthesis

**Verdict: the architecture is sound; the design *prose* bears heresy. Not a
shitshow — but not clean enough to plan as it stands.** The load-bearing idea
(split by cache property: model-agnostic content rides the prefix cache,
model-specific content rides the cache-busting `doctrine_onboard`) is coherent,
consumes SL-186's locked contract correctly in substance, and already survived
one codex pass on the SL-186 carve (§10, F10–F16). The engine reuse posture
(reuse boot's generator, don't rewrite) is right.

The taint is concentrated in **one place, and it is the design's own spine**: the
behaviour-preservation gate is *misstated*. Two `major` charges are confessed
falsehoods, verified against the code, not inferred:

- **F-4 (confirmed under cross-examination).** `doctrine_onboard` drops its
  two-memory load, but `tests/e2e_mcp_server.rs:1083` asserts the
  `"Onboarding Memories"` section exists. The change *breaks* that test — so §9's
  "onboard suite stays green unchanged" is false. A contract subtraction dressed
  as an additive extension.
- **F-2.** boot's Onboarding footer is *in* the section table (`boot.rs:104-132`)
  and rendered by a dedicated `Footer` arm (`boot.rs:292-297`); D3 removes it and
  substitutes an inline-memory section. "Assembly logic untouched" is false.
  Removal is not additive.

Two more `major` charges are genuine underspecifications the design hand-waves:

- **F-5.** `doctrine_onboard` takes no arguments and its handler receives no
  model id (`tools.rs:327-333, 409-414`); "read the client's model" names a seam
  that does not exist. Only the "offer `model-keys`, agent self-identifies" half
  is real.
- **F-6.** `collect_all` (`memory.rs:2736`) *silently* skips shipped uid-dupes —
  its own comment says so — so INV-D3's "local-wins is observable" is false; and
  its order is unsorted fs iteration, so "deterministic key order" requires the
  design to impose the sort (the proven `boot_keys` `key-else-uid` pattern at
  `memory.rs:2830` is the model, uncited).

Three lesser taints (`minor`): F-1 (contract paraphrase narrows axis-invariance
to a subset of axes — INV-D1 self-contradiction risk), F-3 (tier-2 disk fallback
stale-window tolerance left implicit), F-7 (OQ-1 byte budget left a bare open
question rather than a settled posture).

**Ordered penance (all land on `design.md` — this is design reconciliation, not
code):**
1. F-2, F-4 — stop calling the boot + onboard changes "additive / suites green
   unchanged." Name them intentional contract changes; §9 must budget for the
   changed boot goldens **and** the changed onboard e2e assertion.
2. F-5 — cut the fictional "read the client's model"; commit to the
   self-identification mechanism.
3. F-6 — specify order as `key else uid` (reuse `boot_keys`); add a collision
   diagnostic or drop the "observable" claim from INV-D3.
4. F-1, F-3, F-7 — tighten the contract quote; make the tier-2 tolerance an
   explicit decision; promote OQ-1 to a decision.

**Standing risk if planned as-is:** the plan would inherit acceptance criteria
(§9) that the implementation is guaranteed to violate — the behaviour gate would
fail on first green run, and a planner trusting "suites green unchanged" would
mis-scope the test churn. Reconcile the design, re-lock, *then* plan.

> **HERESIS URITOR; DOCTRINA MANET**
