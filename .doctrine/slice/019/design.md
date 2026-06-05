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
*old committed* template — the uncommitted 8-section edit is invisible to it. Two
non-obvious traps (both VERIFIED by the SL-019 inquisition):

- **Re-embed trap (CHARGE II).** There is no `build.rs` / `rerun-if-changed` for
  `install/`, and rust-embed uses `debug-embed`. A lone template edit + plain
  `cargo build` reports `Finished` but does **not** re-embed — the binary keeps
  the old template. The embedding crate must be forced to recompile:
  `touch src/install.rs && cargo build` (or `cargo clean -p doctrine`).
- **Path trap (CHARGE I).** Do not hardcode a binary path. The cargo
  `target_directory` is redirected out of the repo
  (`/home/david/.cargo/doctrine-target-jail`); `./target-jail/` and
  `~/.cargo/bin/doctrine` are *stale* copies carrying the old template. Resolve
  the fresh binary via `cargo metadata --format-version=1` (`target_directory`)
  or just invoke `cargo run -- spec …`.

Entry ritual for PHASE-02 (the gate — never trust `Finished` as proof of embed).
Note only the **template** is embedded by `spec new`; the skill is *not* embedded
and its rework is exemplar-driven, so it lands in PHASE-03, not here.

1. Apply the §4 reword (§10) and commit the template edit (CHARGE III — a dirty
   working-tree template is not a "fixed target"). The skill stays in-flight
   until PHASE-03.
2. Force a re-embedding rebuild: `touch src/install.rs && cargo build`.
3. **Verify the embed:** scaffold a throwaway product spec; `spec show` MUST
   contain `## 1. Intent` and MUST NOT contain `## Problem`. Only then author
   real specs (via `cargo run -- spec …` or the `cargo metadata`-derived binary).

A spec scaffolded with a stale binary carries the old
`Problem/Value/Outcomes` headings — a silent corpus-wide defect. `spec validate`
will **not** catch it (it checks FK integrity, not prose), so the grep gate is
the only guard.

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
- **§7 Verification coverage table — DECIDED (CHARGE IX).** No per-requirement
  coverage table in the corpus: `verification.coverage@v1` is deferred in the
  entity model (`doc/spec-entity-spec.md`), and a prose table keyed on the
  *mobile* `FR-`/`NF-` labels is fragile under relabel. §7 prose carries the
  verification *approach* (what proves the capability, observability, gates); any
  unavoidable requirement reference uses the durable `REQ-NNN`, never the label.
  The reworked skill's §7 and the exemplar both follow this.
- **Template §4 guidance line vs D-1 — DECIDED (CHARGE V): reword (option a).**
  The §4 body line "Functional requirements, non-functional requirements,
  constraints, and invariants" (install/templates/spec-product.md:12–13) invites
  the prose-FR/NF double-storage D-1 forbids (`spec validate` won't catch it). It
  is reworded to: **"Constraints and invariants. (Functional/quality requirements
  are `REQ` entities — add via `spec req add`.)"** This is a sanctioned one-line
  *guidance* clarification, not a structural re-edit — the eight section headings
  are unchanged. The edit lands at PHASE-02 entry, committed with the template +
  reworked skill, before the re-embedding rebuild.
- **Double "Requirements" heading** in `spec show` (prose §4 + synthesized) is
  accepted; the exemplar may title its §4 "Constraints & Invariants" to avoid
  it, at the cost of diverging from the template's section label.

## 11. Plan-time gates (from the inquisition)

- **PHASE-02 entry** = §7a ritual passed (committed template with §4 reword;
  re-embedding rebuild; `spec show` grep gate green). Skill rework is PHASE-03.
- **PHASE-03 entry** = PHASE-02 exemplar locked/accepted (the skill rework is
  exemplar-driven — CHARGE VIII).
- **PHASE-05** = grep the committed diff for any taxonomy/source-map artifact
  under `doc/` or `slice/019/` and reject it — the taxonomy stays in the
  gitignored phase sheet / handover (CHARGE VII).
- **Post-slice** = harvest a memory: rust-embed `debug-embed` + no
  `rerun-if-changed` for `install/` ⇒ a lone template edit is invisible until the
  embedding crate is forced to recompile (CHARGE II footgun).
