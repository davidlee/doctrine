# Design SL-144: ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The `install/` ship surface is the entire user-facing knowledge/customisation
surface doctrine copies into every client. It has never been audited as a
**system**: what each document is responsible for, how they cross-reference,
which access-pattern tier governs each, and how an agent discovers them. Since
ADR-005 was accepted (2026-06-08) and this slice was cut (2026-06-23) the surface
drifted — SL-186/187 added the prompt-cascade tier (`hymns/`, `model-band.md`)
and retired the boot-footer round-trip, while new entity kinds (REC/REV/POL/STD/
knowledge) and CLI verbs landed without the reference docs keeping pace.

The design produces **one coherent IA map** of the surface, then executes the
corrections that map implies: retire the dead hook, document the live ones,
bring the reference docs current, enforce the restate line, and make every
shipped doc reachable. It is a documentation/IA slice — no engine, entity, or
CLI architecture changes (ADR-011/SPEC-023 own the hymns mechanism; this slice
cross-references, never amends).

## 2. Current State

`install/` ship surface (live tree, 2026-07-03):

| Path | Role | Access tier | Drift |
|---|---|---|---|
| `routing-process.md` | boot digest: routing table, core process, guardrails, reference-forms | PUSH (boot) | verify R-OQ-5 block current |
| `governance.md` | user governance pointer | PUSH (`boot.rs GOVERNANCE_REL`) | live |
| `model-band.md` | universal model-band floor | PUSH (boot `Static`) | **new**, undocumented as hook |
| `boot-footer.md` | (was) `## Onboarding` source | **retired** (SL-187) | **orphan** — delete |
| `glossary.md` | vocabulary, ids, reference forms, verification taxonomy | PULL-ref | stale kinds/verbs |
| `using-doctrine.md` | which verb for intent, storage tiers, read via `show`, edit rules | PULL-ref | stale kinds/verbs |
| `review-ledger.md` | adversarial review protocol | PULL-ref | check currency |
| `hymns/` | context-keyed prompt cascade (framework site) | PULL cascade (ADR-011/SPEC-023) | map only; SL-191 owns internals |
| `agents/` | subagent defs (e.g. `dispatch-worker.md`, hymn-baked) | build asset | map only |
| `templates/` | seed bodies (`seed-onboarding.md` → orientation memory) | build asset | map only |
| `manifest.toml` | install manifest: dirs, memory seed, gitignore, hymns seal/expose, root markers | reachability oracle | not in IA today |
| `doctrine.just`, `doctrine.toml.example`, `LICENSE` | config/build scaffolding | build asset | out of knowledge-IA |

Onboarding hook (post-SL-187): `manifest.toml [memory]` seeds
`mem.signpost.project.orientation` from `templates/seed-onboarding.md`; the
onboarding-tagged memory body populates the boot `## Onboarding` section. No
`boot-footer.md` read path exists anywhere in `src/`.

Skills: 30 under `plugins/doctrine/skills/*/SKILL.md`; 8 carry `--flag <ARG>`
candidates for restate-line triage (backlog, close, worktree, dispatch,
dispatch-agent, dispatch-subprocess, reconcile, reviewing-memory).

## 3. Forces & Constraints

- **ADR-005** — the governing tiering: PUSH carries load-bearing rules only;
  PULL-reference docs must be *pointed-at* to be reachable; skills route, don't
  restate (R-OQ-4 restate line); no shipped doc duplicates `doctrine --help`.
- **ADR-011 + SPEC-023** — govern the hymns cascade. This slice maps, never
  amends; if the cascade needs folding into ADR-005's tiers, that is a review
  finding, not a mid-slice amendment.
- **POL-002** — platform independence: docs carry no host-project convention as
  correctness.
- **STD-001 / STD-002** — named constants (embed roots, marker strings), short
  entity titles, ids not slugs.
- **Evidence-bound (R-C1)** — restate-line fixes target named offenders, not a
  blanket rewrite.
- **Re-embed footgun** — `install/*` edits need `touch src/install.rs` +
  `cargo build` to ship; batch edits.
- **Scope shape C+** — audit `.md` + manifest as the system; map the new subdirs
  at stable altitude without restructuring SL-186/187/191 territory.

## 4. Guiding Principles

- **One responsibility per doc; one home per fact (DRY).** Every overlap
  resolves to a single owner + a pointer from the others.
- **Access pattern decides tier.** A rule needed *before* any skill invoke →
  PUSH; shared detail → PULL-reference; context-keyed → hymns cascade.
- **Reachable or retired.** A doc not in the ship set and not pointed-at is dead
  weight — delete it (boot-footer) rather than leave residue.
- **Map, don't rebuild.** The new subdirs are accounted-for in the IA with
  governance cross-refs; their internals stay with their owning slices.

## 5. Proposed Design

### 5.1 System Model

The IA is a **four-tier access model** over the ship surface. Each tier answers
"when does an agent encounter this content?" — the ADR-005 thesis extended with
the ADR-011 cascade as a first-class, separately-governed tier.

```
TIER 0 — PUSH (boot-resident, every session, no invoke)
  routing-process.md   workflow digest + guardrails + reference-forms (R-OQ-5)
  governance.md        user governance pointer      [boot GOVERNANCE_REL]
  model-band.md        universal model-band floor   [boot Static]
  (onboarding memory)  seeded from templates/seed-onboarding.md → boot Onboarding
        │  carries LOAD-BEARING rules only; points at Tier 1 for detail
        ▼
TIER 1 — PULL-reference (shipped docs; read when Tier 0 or a skill points)
  glossary.md          vocabulary · ids · reference forms · verification taxonomy
  using-doctrine.md    which verb for intent · storage tiers · read-via-show · edit rules
  review-ledger.md     adversarial review protocol
        │  each MUST be pointed-at (skill or boot) AND in the manifest ship set
        ▼
TIER 2 — Skills (thin routers, plugins/doctrine/skills/*/SKILL.md)
        │  name a verb, cite a rule by name; never reproduce flags/tables (R-OQ-4)
        ▼
TIER 3 — Hymns cascade (context-keyed PULL; ADR-011/SPEC-023) — MAPPED, not owned
  install/hymns/ (framework) + .doctrine/hymns/ (user overlay)
  resolved by `doctrine prompt resolve` on (harness, model, role, stage, project)

REACHABILITY ORACLE — manifest.toml
  [dirs] [memory] [gitignore] [hymns seal/expose] [root_markers]
  a doc ships iff install.rs embed-copies it; a hymn ships iff manifest exposes it
```

**Responsibility boundaries** (resolves the overlaps Objective 1 hunts):

- `routing-process.md` = *WHEN/route* + the handful of load-bearing rules
  (reference forms, storage tiers, "use the CLI"). It **points at** Tier 1 for
  the *how*; it never carries verb-by-intent tables or edit mechanics.
- `using-doctrine.md` = *which verb for which intent* + storage-tier read/write
  + edit-preserving rules. The single home for CLI/editing mechanics prose.
- `glossary.md` = *vocabulary*: kinds, id/reference forms, verification taxonomy.
  Definitional only — no workflow, no verb-routing.
- `review-ledger.md` = the review protocol; owned by the review/inquisition path.

Boundary test: if two docs state the same fact, one becomes the owner and the
others cite it by name. If a fact has no home, it lands in the tier its access
pattern dictates.

**Cross-reference edges** (the reachability graph the audit must close):
- Tier 0 → Tier 1: routing-process points at glossary + using-doctrine.
- Skills → Tier 1: skills that need the *how* cite using-doctrine / glossary
  by name (not by reproducing it).
- manifest → every shipped doc: the ship set must list each Tier 0/1 doc;
  an unshipped doc is unreachable regardless of pointers.

**Governance partition:** Tiers 0–2 are ADR-005's domain (this slice's audit
target). Tier 3 is ADR-011/SPEC-023's domain (mapped, cross-referenced, not
restructured). The IA map names both so the surface is fully *accounted for*
without the slice reaching into cascade internals.

### 5.2 Interfaces & Contracts

### 5.3 Data, State & Ownership

### 5.4 Lifecycle, Operations & Dynamics

### 5.5 Invariants, Assumptions & Edge Cases

## 6. Open Questions & Unknowns

## 7. Decisions, Rationale & Alternatives

## 8. Risks & Mitigations

## 9. Quality Engineering & Validation

## 10. Review Notes
