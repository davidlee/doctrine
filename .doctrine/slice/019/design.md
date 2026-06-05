# SL-019 — Backfill Doctrine product-spec corpus · design

## 1. Frame

The product-spec corpus is empty. SL-015 shipped the spec entity machinery and
the product template was restructured into eight sections. This slice authors
the full product-spec corpus for Doctrine's *own* capabilities — dogfooding the
spec machinery — with agents authoring from the existing material (`doc/*`,
slices, ADRs, `src/`, memory).

Locked decisions (from scoping + design Q&A):

- **Full backfill**, Doctrine's own capabilities, agents authoring.
- **Approach:** taxonomy (scaffolding) → exemplar + exemplar-driven SKILL.md
  reconciliation → backfill fan-out → validate.
- **Exemplar = Slices** (the spine capability; richest what/why).
- **Composition model** (D-1) and **skill reconciliation** (D-2) below.

## 2. Composition model (D-1) — the foundational decision

`doctrine spec show` reassembles a product spec as (src/spec.rs:324–423):

```
`PRD-NNN` — <title>            identity + flat fields (from spec-NNN.toml)
<prose body, VERBATIM>          the 8-section template md
## Requirements                 SYNTHESIZED from membered REQ-NNN peer entities
  ### FR-001 (REQ-007) — <title>
  <statement · acceptance criteria>
```

The requirements section is **synthesized from entities**, not read from prose.
Therefore:

- **Functional (FR) and quality (NF) requirements live ONLY as `REQ-NNN` peer
  entities**, added via `doctrine spec req add <PRD-ref> --kind functional|quality`.
  They are never written as prose rows in the md body. Writing them in both
  places reintroduces the double-storage drift the entity model exists to kill
  (`doc/spec-entity-spec.md` § Diagnosis).
- **Template §4 "Requirements" prose carries constraints, invariants, and
  framing commentary only** — material that has no requirement *kind* in the
  CLI (`--kind` is `functional|quality` only; there is no `constraint`/
  `invariant` kind). The itemized FR/NF rows are entities.
- **Labels are `FR-` / `NF-`** (the CLI's quality label is `NF-`, not `NFR-`).
- `spec show` output will show two requirement-flavoured headings — the prose
  `## 4. Requirements` (constraints/invariants) and the synthesized
  `## Requirements` (FR/NF entities). Accepted: they carry different content.

Consequence: a correctly-authored product spec is *three* coordinated writes —
the identity toml (flat fields: `title`, `slug`, `status`, `tags`,
`responsibilities[]`, optional `category`), the 8-section md body, and a set of
`spec req add` calls. `spec validate` proves coherence: no orphan requirements,
no dangling member FKs, no duplicate labels.

## 3. Skill reconciliation (D-2)

Canonical source: `plugins/doctrine/skills/spec-product/SKILL.md` (install
propagates to the `.doctrine/skills/` and `.claude/skills/` copies). The
freshly-expanded skill currently conflicts with D-1 in three places:

1. **§4 example prescribes prose FR/NFR rows** — collides with REQ entities.
2. **Internal contradiction** — its Structural Doctrine section says "Do not
   duplicate canonical requirement content into narrative prose when it belongs
   as a requirement entity," contradicting its own §4 example.
3. **Label drift** — writes `NFR-`; the CLI emits `NF-`.

The current skill was authored by a web agent **blind to the recent spec-entity
code and changes** — so PHASE-03 is a **full rework** (user granted latitude),
not a surgical patch. Goal: a skill that matches the as-built model and is
**exemplar-driven**:

- Author requirements as `spec req add` entities (surfaced by `show`); §4 prose
  reserved for constraints/invariants/commentary — kill the prose-FR/NFR example.
- `NFR-` → `NF-`; state that `CON-`/`INV-` have no entity kind and stay prose.
- Remove the internal prose-rows contradiction.
- Ground every command/structural claim in the actual CLI (`spec new/req add/
  show/validate/list`) and the three-write authoring reality (toml + md + reqs).
- Point at the locked exemplar (PRD for Slices) as the canonical shape.
- Preserve the genuinely-good additions the web agent got right (boundary,
  review-defect checklist, handoff) where they don't conflict with the model.

## 4. Taxonomy method (scaffolding — not committed)

The taxonomy and its source map are **disposable runtime context** (handover /
phase sheet), consumed by the backfill — not a persisted authored artifact.

- **Unit = a product capability:** a user-facing value with its own what/why.
  Maps roughly to a top-level CLI noun and/or a `doc/*` capability spec; the
  mapping is *not* 1:1 with `doc/*` (those skew toward the *how*).
- **Excluded as internal mechanism (the *how*, not product):** entity-model,
  relation-index, the reservation primitive, glossary. These are *sources* for
  the capability specs, not capabilities themselves.
- **Candidate capabilities (~7), to confirm in PHASE-01:** slices ·
  specifications · memory · ADRs · boot/governance snapshot · install · skills.
  (drift ledger / reservations are design-note / primitive material — PHASE-01
  decides whether either rises to a product capability or stays internal.)
- **Source map:** per capability, the `doc/*` + CLI surface + ADRs + memories
  that feed its what/why. Drives each authoring task; lives in the phase sheet.

## 5. Exemplar — PRD for Slices

Authored end-to-end as the reference bar:

- **Identity toml:** title "Slices", slug, `status = "draft"`, `responsibilities[]`
  drawn from `doc/slices-spec.md`.
- **§1 Intent:** scope intentional change into shippable units with a governed
  lifecycle. §2 Scope: what slices govern vs ADRs (project-global) vs specs
  (evergreen). §3 Principles: ask-don't-infer, the storage rule, no-code-without-
  plan, immutable phase/criteria ids. §4: constraints/invariants (e.g. "a slice
  is one shippable change"; "authored vs runtime vs derived storage tiers").
  §5 Success Measures, §6 Behaviour (new→design→plan→phase→execute→audit→close),
  §7 Verification, §8 Open Questions.
- **Requirements (entities):** FR for the lifecycle operations; NF for
  auditability / storage-tier integrity. Added via `spec req add`.
- Gate: `spec validate` green for PRD-Slices; `spec show` reviewed and accepted
  as the bar before fan-out.

## 6. Backfill fan-out

Agents author the remaining capabilities in parallel, one PRD per agent, each
following the exemplar + reconciled skill + its source-map entry.

- **Collision safety:** each PRD is its own entity tree (`spec/product/<n>/`) —
  no file collisions. `spec req add` reserves `REQ-NNN` via the atomic `mkdir`
  claim with a bounded retry loop (`entity.rs` `allocate_fresh`,
  `MAX_CLAIM_RETRIES`); lost races recompute and retry. The only failure mode is
  retry exhaustion under heavy concurrent reservation — mitigated by **bounding
  fan-out width** (≤ a few concurrent authors; the corpus is ~6 specs).
- **Execution mechanism** (decided at `/phase-plan`/`/execute`): the parallel
  authoring is a candidate for the Workflow harness ("agents backfill" = the
  user's opt-in), or serial `/execute`. Not a design-locked choice.
- Each authored spec is *what/why*, durable, implementation-agnostic; the *how*
  stays in `doc/*` / `/spec-tech`.

## 7. Validation / closure

- `doctrine spec validate` clean **corpus-wide**: no dangling member FKs, no
  duplicate labels within a spec, no orphan requirements.
- `doctrine spec show <PRD>` reassembles each spec cleanly.
- Every capability in the confirmed taxonomy has an authored PRD following the
  exemplar's shape (what/why, not how).
- Reconciled skill no longer conflicts with D-1.
- `just check` green; storage rule honoured (no queried/derived data in prose;
  taxonomy/source-map not committed).

## 7a. Build prerequisite (blocking)

Templates are **embedded at build time** (`rust_embed`, `#[folder = "install/"]`
src/install.rs:17). The installed ro `~/.cargo/bin/doctrine` was built from the
*old committed* template — the uncommitted 8-section edit is invisible to it.
Therefore, before any spec is scaffolded:

1. Commit the template edit (and the reworked skill).
2. `cargo build`.
3. **Author with `./target/debug/doctrine`** (carries the new embedded template),
   not the stale `~/.cargo/bin/doctrine`.

This is the entry condition for PHASE-02. A spec scaffolded with the stale binary
would carry the old `Problem/Value/Outcomes` headings — a silent corpus-wide
defect.

## 8. Phase shape (provisional — for `/plan`)

1. **PHASE-01 Taxonomy** — confirm the capability set + source map (scaffolding).
2. **PHASE-02 Exemplar** — author PRD-Slices end-to-end; lock the bar.
3. **PHASE-03 Reconcile skill** — exemplar-driven SKILL.md fixes (D-2); must
   precede fan-out so agents read corrected guidance.
4. **PHASE-04 Backfill** — fan-out author the remaining PRDs.
5. **PHASE-05 Validate** — corpus-wide `spec validate` + `spec show` + coverage
   audit against the taxonomy.

## 9. Risks

- **Source skew** — `doc/*` is mostly the *how*; distilling *what/why* needs
  judgement. The exemplar sets altitude; review gates it.
- **Parallel drift** — concurrent authors diverge in shape/voice. Mitigated by
  locking exemplar + skill before fan-out.
- **Skill was authored blind** — the web agent had no view of the spec-entity
  code; PHASE-03 reworks it in full against the as-built model rather than
  patching, salvaging the parts that are right.
- **Reservation contention** — bounded retry loop; mitigated by capped fan-out
  width.
- **Taxonomy boundary calls** — drift ledger / reservations / entity-model:
  capability vs mechanism is a PHASE-01 judgement, not predetermined.

## 10. Open questions

- Final capability count + the drift/reservations boundary call (→ PHASE-01).
- Execution mechanism for the fan-out: Workflow vs serial `/execute`
  (→ `/phase-plan`).
- **§7 Verification coverage table** (skill prescribes `| Requirement | … |`):
  should it reference durable `REQ-NNN` or the mobile `FR-`/`NF-` label?
  Referencing labels in committed prose is fragile under relabel. Resolve in the
  exemplar (PHASE-02) — likely reference behaviour/durable id, or omit the
  per-requirement table (coverage@v1 is deferred in the entity model).
- **Double "Requirements" heading** in `spec show` (prose §4 + synthesized) is
  accepted; the exemplar may title its §4 "Constraints & Invariants" to avoid
  it, at the cost of diverging from the template's section label.
