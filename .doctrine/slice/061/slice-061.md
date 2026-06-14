# Rewire /code-review and /inquisition onto the RV review ledger

## Context

ADR-007 D-C0 makes adversarial review a first-class kind (RV, `RV-NNN`): one
generic, facet-parameterized ledger that every review skill instantiates. SL-040
shipped the kind and the `doctrine review` verb family; IMP-001 piloted the
rewiring on `/audit` (now opens a `reconciliation`-facet RV instead of authoring
the hand-made `audit.md`). The remaining review skills still produce unstructured
output: `/code-review` emits free-form prose findings, `/inquisition` writes
`inquisition.md`. Neither benefits from the ledger's append-only finding
identity, turn-graph dispositioning, severity/close-gate teeth, or the warm
reviewer-context cache.

This slice rewires the two review skills whose work is genuinely still
unstructured onto the RV ledger, completing the lifecycle coverage ADR-007 D-C0
envisaged.

## Scope & Objectives

- **`/code-review` → RV.** The `code-review` facet already exists in the review
  kind's facet enum. Rewrite the skill (in the `review` plugin) to open a
  `code-review`-facet RV against the subject, raise each finding as a structured
  ledger entry with the existing severity vocabulary, and render its synthesis +
  haiku into the review's `## Synthesis`. Preserve the skill's voice and the
  existing severity-label mapping (`🔴 blocking → blocker`, etc.).
- **`/inquisition` → RV.** Rewire the heresy-hunt onto the ledger so charges
  become append-only findings and sentencing becomes dispositioning. The
  inquisition's facet binding is an **open question** (see below) — it may reuse
  an existing facet, or motivate a new one. Preserve the skill's adversarial
  posture and output voice.
- Keep the rewiring **DRY** against the `/audit` pilot — the RV-driving prose
  (open → prime → raise → dispose/verify → synthesis) is shared shape; lift the
  common pattern rather than copy-paste three divergent variants.

## Non-Goals

- **`drift`** — explicitly **not** an RV facet (ADR-007 D-C11 carves multi-artefact
  drift into a distinct future **Drift Ledger** kind). Owned by **IMP-022**; the
  IMP-023 title predates the D-C11 carve-out. Out of scope here.
- **`reconciliation`** — already rewired (the IMP-001 `/audit` pilot). Untangling
  reconciliation into its own skill / the audit-reconcile seam is **IMP-008**. No
  reconciliation work here.
- **Fork-invoked / parallel-raiser review** (IMP-024) and **RV verb e2e golden
  coverage** (IMP-029) — adjacent, separately owned. This slice rides the existing
  single-tree, parent-locus verb surface as-is.
- No new RV verbs or coordination mechanics. If `/inquisition` needs a new facet,
  that is a bounded enum + validation change, not a protocol change.

## Affected Surface

- `plugins/review/skills/code-review/SKILL.md`
- `plugins/doctrine/skills/inquisition/SKILL.md`
- Possibly the facet enum + its validation (only if the inquisition open question
  resolves to a new facet): `src/` review/facet definitions and the
  `review new --facet` help/enum.
- Skill re-embed path on any SKILL.md change (`doctrine claude install` + re-embed
  seam) — see memory on skill refresh.

## Risks, Assumptions, Open Questions

- **OQ-1 — inquisition facet.** Does `/inquisition` map onto an existing facet
  (`implementation`? `design`?), pick the facet by what it targets, or motivate a
  new `inquisition` facet in the enum? Inquisition is a cross-cutting posture, not
  a lifecycle stage — the mapping is not obvious. Resolve in `/design` (likely a
  `/consult`).
- **OQ-2 — slice-less / diff-only targets.** `/code-review` and `/inquisition`
  frequently run against a raw diff or PR with no governing entity, but
  `review new --target` requires a canonical ref (ADR-007 D-C11 single-ref
  subject). What is the target when there is no slice/phase/spec to point at? This
  may bound how much of these skills can move onto RV, or motivate a target
  convention. Resolve in `/design`.
- **OQ-3 — one slice or split.** `/code-review` (existing facet, mostly
  skill-prose) and `/inquisition` (needs a facet decision, possible CLI change)
  differ in altitude. Keep as one slice with two phases, or split if the
  inquisition CLI work sprawls. Decide at planning.
- **ASM** — the existing `review` verb family is sufficient for both skills'
  finding lifecycle (raise/dispose/verify/contest/withdraw); no new verb needed.

## Verification / Closure Intent

- Both SKILL.md files drive the `doctrine review` ledger end-to-end (open → raise
  → dispose/verify → synthesis), mirroring the `/audit` pilot, with their distinct
  voices intact.
- The severity vocabulary each skill uses maps onto the RV `blocker|major|minor|nit`
  axis (and the close-gate teeth) coherently.
- If a new facet is introduced, it is in the enum, validated, and covered by a test
  in line with the existing facet handling.
- Skills re-embed and load; a smoke review (open an RV via each skill's flow,
  raise + dispose a finding) succeeds.
- IMP-023 backlog item updated to reflect the drift/reconciliation reassignment and
  closed when the two in-scope skills land.

## Follow-Ups

- Tracked separately: IMP-022 (drift ledger), IMP-008 (reconcile seam), IMP-024
  (fork/parallel-raiser), IMP-029 (RV verb e2e goldens).
