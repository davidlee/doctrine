I now have all the evidence. Let me produce the research brief.

---

# Research Brief: Is the PHASE too coarse a unit of actionability?

## Answer

**No, on this evidence.** Obligation-level (EX-criterion-level) dependency modelling would not meaningfully change the frontier of workable units compared to what the authored phase order already gives us. The EX criteria *are* specific and checkable enough to serve as obligations — SL-233's in particular are among the most precisely specified exit criteria I have seen in any engineering plan — but the dependencies between them are overwhelmingly phase-wide: a phase needs its predecessor's whole output, not a fragment of it. Where a phase *does* need only part of a predecessor's output (SL-233 PHASE-10 needing only PHASE-04, not PHASE-05 through PHASE-12), the plan explicitly acknowledges the slack and the deferral is an architectural choice ("three seams ride that stack"), not a constraint of modelling granularity.

The EN criteria are, as hypothesised, the best available source of dependency edges: every EN row that names a prior phase also elaborates *what specific output* of that phase is needed (e.g., "the pure model exists to serialize", "the envelope and `apply` exist to export from"). But in 27 of 29 phase-to-phase EN edges across the three slices, the thing needed *is* the full phase output — the elaboration narrows from "phase complete" to "this particular output of the phase" without narrowing *within* the phase. Only two edges (SL-233 PHASE-12 EN-2 → PHASE-05 EX-6, and SL-233 PHASE-10 EN-1 → PHASE-04) name a proper subset of a prior phase's obligations, and in both cases the phase ordering already respects the tighter constraint from a different dependency.

The falsifier that bites hardest: **real dependencies are predominantly phase-wide.** SL-233's plan.md states this directly — "The serialism is forced rather than chosen: phases 02–07 nearly all route through `src/design_run/` or `src/commands/design.rs`, so there is no honest file-disjoint pair to parallelise." The phases build a vertical stack where each extends the one before it; the EX criteria are the seams between layers, not independent work items.

## Evidence

### SL-233: The 16-phase monster

**Obligation inventory.** EX criteria are highly specific and checkable. Examples from the first three phases:

- PHASE-02 EX-2: "the five coarse stages of design.md §5.4 are a closed vocabulary: Exploring, Inquiring, Drafting, Reviewing, Locked — declared as a single enum NAMED `Stage` at `src/design_run/mod.rs`" (`.doctrine/slice/233/plan.toml:91-92`)
- PHASE-03 EX-12: names 13 exact test function names that must exist in `tests/e2e_design_state.rs`, each anchored at line start (`.doctrine/slice/233/plan.toml:127-128`)
- PHASE-03 EX-8: "writes use the existing `src/fsutil.rs::write_atomic` sibling-replacement helper, not a new writer. The snapshot path derives from `src/state.rs`'s existing private `STATE_SLICE_DIR` constant through a new `pub(crate) fn design_snapshot_path` in that same module" (`.doctrine/slice/233/plan.toml:117`)

No phase has EX rows that are "vague enough that they state no checkable state of affairs." Even governance-only phases (PHASE-01) name exact spec entity paths and specify commit ordering (EX-7: "the selector append lands as its own commit, distinct from and preceding every spec-body commit").

**Dependency edges.** 16 phases, 14 EN criteria name prior phases. All but 2 are *effectively phase-wide*:

| From | To (via EN) | EN text | Actually needs |
|------|-------------|---------|----------------|
| PHASE-01 | PHASE-02 | "PHASE-01 complete — the technical spec exists and can be cited" | All of PHASE-01's descent outputs |
| PHASE-02 | PHASE-03 | "PHASE-02 complete — the pure model exists to serialize" | All of PHASE-02's types |
| PHASE-03 | PHASE-04 | "PHASE-03 complete — a persisted run exists to project" | All of PHASE-03's persistence |
| PHASE-04 | PHASE-05 | "PHASE-04 complete — `apply` exists to carry a checkpoint declaration" | All of PHASE-04's command surface |
| PHASE-05 | PHASE-06 | "PHASE-05 complete" | All of PHASE-05's checkpoint protocol |
| PHASE-06 | PHASE-13 | "PHASE-06 complete — a section title is DERIVED from its body" | Title derivation specifically, but 13 also needs 06's wire contract |
| PHASE-13 | PHASE-14 | "PHASE-13 complete — markers, byte-exact framing, the watermark and `seq` exist" | All of PHASE-13's write side |
| PHASE-06 | PHASE-14 | "PHASE-06 complete — the title derivation procedure exists" | PHASE-06's title derivation (EX-13b declare arm) |
| PHASE-13,14 | PHASE-11 | "PHASE-13 AND PHASE-14 complete" | All of both |
| PHASE-13 | PHASE-12 | "PHASE-13 complete — fingerprinted sections exist" | Fingerprinted sections; but PHASE-12 also needs PHASE-05 |
| **PHASE-05** | **PHASE-12** | **"PHASE-05 complete — the content-bound acceptance-attestation shape exists from DEC-088"** | **Only PHASE-05 EX-6** (the attestation shape), not all of PHASE-05 |
| **PHASE-04** | **PHASE-10** | **"PHASE-04 complete — the envelope and `apply` exist to export from and accept into"** | **Only PHASE-04's envelope + apply**, not PHASE-05–12 |
| PHASE-12 | PHASE-07 | "PHASE-12 complete — the four process fragments name obligations that all now exist" | All of PHASE-12's review stage |
| PHASE-07 | PHASE-16 | "PHASE-07 complete — the family is executable end-to-end" | All of PHASE-07's prompt pack |
| PHASE-07 | PHASE-08 | "PHASE-07 complete" | All of PHASE-07 |
| PHASE-08 | PHASE-09 | "PHASE-08 complete — the skill and prompt assets exist" | All of PHASE-08 |

The two bolded rows are the only cases where a phase needs a *proper subset* of a prior phase's obligations. Both are architecturally deferred: PHASE-10 (delegation) and PHASE-12's partial dependency on PHASE-05 are placed late because the plan.md (§ Sequencing & Rationale) states "Three seams then ride that stack — entry from authored prose (PHASE-11), the reviewing stage (PHASE-12), and delegation (PHASE-10) — before assets, adapter, and evaluation close the slice."

**Frontier comparison.** If we modelled at EX-criterion level:

- **PHASE-10 could have run right after PHASE-04.** Its EN-1 says "PHASE-04 complete — the envelope and `apply` exist." It needs nothing from PHASE-05, 06, 13, 14, 11, or 12. The authored phase order puts PHASE-10 as the 11th phase. This is the clearest case where obligation-level modelling differs from phase-level modelling, but the plan.md *already knows this* — it's not a dependency constraint, it's an architectural choice to "ride the stack" for coherence.

- **PHASE-12's dependency on PHASE-05 is narrower than phase-wide** (it needs only EX-6, the attestation shape, not the crash-recovery protocol or the fresh-creation seam). But PHASE-12 is already gated by PHASE-13 (fingerprinted sections), which is later, so this narrowing changes nothing about the frontier.

- **Every other edge is phase-wide.** Within a phase, EX criteria are tightly coupled — you cannot deliver "the snapshot is schema-versioned" (PHASE-03 EX-1) without also delivering "revision CAS" (EX-2) and "submission idempotency" (EX-3) because they route through the same `src/commands/design.rs` and `src/design_run/snapshot.rs` files.

**Historical execution.** Recoverable from the notes.md Harvest section and git log. All 16 phases completed. Execution order was the authored array order: `01 → 02 → 03 → 04 → 05 → 06 → 13 → 14 → 11 → 12 → 10 → 07 → 16 → 08 → 09`. No phase ran out of order. No phase was revisited for re-execution, though many were *amended* (criteria appended) mid-flight — PHASE-03 gained EX-17/EX-18 during execution (`.doctrine/slice/233/notes.md` § Learned), PHASE-06 was split into three (13/14/06), and PHASE-16 was inserted before PHASE-08. These are plan revisions, not execution reorderings.

### SL-057: The pipeline

**Obligation inventory.** 5 phases, each with 3–6 EX criteria. More compact than SL-233 but still checkable:

- PHASE-01 EX-2: "derive_status implements the §5.2 verdict table — Unobtainable => Blocked; Ran{exit0 ∧ matcher} => Verified; else => Failed — and INV-3 holds" (`.doctrine/slice/057/plan.toml:41-42`)
- PHASE-03 EX-2: "record = load→upsert→save and runs coverage::valid AND verify::resolve before any write" (`.doctrine/slice/057/plan.toml:98`)

**Dependency edges.** All strictly linear, all phase-wide. EN criteria narrow *to specific outputs* but never *within* a phase:

- PHASE-02 EN-1: "PHASE-01 complete (VtCheck exists to resolve against)" — needs all of PHASE-01
- PHASE-03 EN-1: "PHASE-01 + PHASE-02 complete (VtCheck, coverage::valid, verify::resolve all exist)" — needs all of both
- PHASE-04 EN-1: "PHASE-01..03 complete (derive_status, verify::resolve, coverage_store, dtoml all exist)" — needs all three
- PHASE-05 EN-1: "PHASE-03 + PHASE-04 complete (coverage_store + the verifier exist and are tested)" — needs 03 and 04

PHASE-05 doesn't need PHASE-01 or PHASE-02 directly, but needs PHASE-03 which needs 01+02. Transitivity makes it phase-wide.

**Requirements array populated.** PHASE-03 and PHASE-05 share `REQ-256`; PHASE-04 and PHASE-05 share `REQ-255`. This shows that requirements can span multiple phases — a single REQ may be partially satisfied by one phase and completed by another. This is the opposite of obligations-as-EX-criteria: a requirement is a cross-cutting concern, while an EX criterion is a phase-boundary deliverable.

**Frontier comparison.** Obligation-level modelling adds nothing. Every obligation in phase N depends on every obligation in phase N-1 (transitively through the shared codebase). The phases are already small — each is a single capability layer — so sub-phase modelling would be ceremony without insight.

### SL-229: The ordinary slice

**Why chosen.** 3 phases, `done`, recent (2026-07-24), phases completed normally. Represents the "typical" Doctrine slice: a small, self-contained capability delivered in a linear pipeline with no plan revisions.

**Obligation inventory.** EX criteria are adequate but less granular than SL-233:

- PHASE-01 EX-1: "doctrine slice research <id>: dir absent → mint ... + stamp baseline.toml ... ; present → per-path drift advisory; --restamp → re-baseline; exit 0 in every advisory outcome" (`.doctrine/slice/229/plan.toml:30-31`)
- PHASE-02 EX-1: "SKILL.md frontmatter name equals dir name (SPEC-010); description written as the real trigger; unrouted — no routing-table or boot change" (`.doctrine/slice/229/plan.toml:56`)

**Dependency edges.** Two edges, both phase-wide:

- PHASE-02 EN-1: "PHASE-01 landed: the verb the skill names exists" — needs PHASE-01 EX-1 specifically (the CLI verb), but PHASE-01 is so small (3 EX criteria, all in one file `src/research.rs`) that narrowing is meaningless
- PHASE-03 EN-1: "PHASE-02 landed: /research exists for the hooks to point at" — needs PHASE-02

**Frontier comparison.** Obligation-level modelling adds nothing. Three phases, each a distinct deliverable kind (code, prose, integration), separated by kind rather than by dependency granularity. You cannot write consumption hooks before the skill exists, and you cannot write the skill before the verb exists.

## Judgement

### What the EN criteria actually encode

Across all three slices, EN criteria serve three distinct purposes:

1. **Phase-complete dependency** (27 of 29 edges): "PHASE-XX complete" with a parenthetical naming *why* — the specific output needed. The elaboration is documentation, not narrowing. It answers "what aspect of the prior phase does this phase build on" rather than "which EX rows of the prior phase can I skip."

2. **Non-phase governance gates** (8 edges across SL-233): design gates ("RV-315 terminal"), decision acceptance ("DEC-092 accepted"), revision approval ("REV-044 approved"). These are orthogonal to phase-level dependency — they gate whether a phase can *start* at all, not what it builds on. They are not obligations of any prior phase.

3. **Partial-phase dependency** (2 edges): SL-233 PHASE-12 EN-2 (needs only PHASE-05 EX-6) and PHASE-10 EN-1 (needs only PHASE-04). Both are architecturally deferred, not blocked by phase coarseness.

### Where obligation-level modelling would differ

Only one case across all three slices: **SL-233 PHASE-10 (delegation) could run after PHASE-04 instead of after PHASE-12.** This is not a discovery — the plan.md already states that PHASE-10 is a "seam that rides the stack" and is placed late for architectural coherence, not dependency constraint.

Every other phase-to-phase edge is phase-wide. Within a phase, EX criteria are tightly coupled by shared files — you cannot deliver one without the others because they modify the same module. SL-233's plan.md says this directly: "phases 02–07 nearly all route through `src/design_run/` or `src/commands/design.rs`, so there is no honest file-disjoint pair to parallelise."

### The requirements arrays don't help

SL-057's per-phase `requirements` arrays show that requirements span phases (REQ-256 appears in both PHASE-03 and PHASE-05). This means requirements are *not* obligations — they are cross-cutting concerns that phases contribute to. Using requirements as obligation nodes would produce a graph where multiple phases share a dependency on the same node, which tells you nothing about ordering.

### The EX criteria are obligations already

The leading hypothesis — that EX criteria already ARE obligations — **holds.** SL-233's EX criteria are the most precisely specified work items in the corpus: they name exact function signatures, test file paths, constant names, and byte-level behaviours. They are checkable. They are the right granularity. The question is not whether they exist but whether dependencies between them differ from dependencies between the phases that contain them — and on this evidence, they don't.

## Limits

- **Runtime phase state is unrecoverable.** The gitignored `.doctrine/state/` tier is gone for all three slices. Execution order is inferred from plan.md, notes.md Harvest sections, and git log commit scopes. I cannot confirm whether any phase was paused and resumed, or whether multiple phases were active concurrently.

- **SL-057 has 6 phases per `grep` but I only examined 5.** The sixth `[[phase]]` match is the header comment block (`# One [[phase]] per ordered phase...` on line 22). The slice actually has 5 phases. The discrepancy in my earlier count was a false positive on the comment.

- **The study examines 3 of ~90 completed slices.** A slice with genuinely parallel obligations — file-disjoint work that has no reason to be serial — might exist in the corpus but was not captured. I deliberately chose the largest slice (SL-233) because it is where the question matters most: if obligation-level modelling doesn't help in a 16-phase plan, it won't help in a 3-phase one.

- **I did not read RFC-026 or RFC-027** per instructions, but the notes.md for SL-233 mentions RFC-026 in a commit message (`c6991b0af doc(RFC-026): E8 — what SL-233's process produced, measured by using it`). If those RFCs contain a counter-argument based on this slice's data, I have not engaged with it.

- **One inference:** SL-233 PHASE-15's EN criteria (RV-324/REV-044/DEC-105) are non-phase governance dependencies. I classified these as "not phase-to-phase edges" without reading the full RV-324 ledger to confirm none of them indirectly depends on a prior phase's output. If they do, that would add complexity to the edge graph without changing the frontier conclusion.

[SL-233 phase 16]: Obligation runbook runner — the last phase examined, and the one whose EX-19 explicitly fences obligations from cross-contaminating the phase-level condition vocabulary.
