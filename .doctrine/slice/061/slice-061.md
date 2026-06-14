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
- **Shared `review-ledger.md`** (new, install-wired) owns the mechanical protocol
  (target ladder, verb sequence, severity/disposition vocab, close-gate, harvest).
  All three review skills — `/audit` included — collapse to *persona + lens +
  ledger-pointer + tail* and reference it. `/audit`'s change is a behaviour-preserving
  **refactor** (it is already on RV; only its inline mechanics move to the doc).
- **Relocate `/code-review` into doctrine core** (`plugins/review/skills/` →
  `plugins/doctrine/skills/`) and **retire the standalone `review` plugin** — the
  rewired skill hard-depends on doctrine, so a standalone install is incoherent.
  Drop its non-doctrine bimodal branch (it is doctrine-native).

## Non-Goals

- **`drift`** — explicitly **not** an RV facet (ADR-007 D-C11 carves multi-artefact
  drift into a distinct future **Drift Ledger** kind). Owned by **IMP-022**; the
  IMP-023 title predates the D-C11 carve-out. Out of scope here.
- **`reconciliation` mechanics** — already rewired (the IMP-001 `/audit` pilot). No
  reconciliation *behaviour* change here. `/audit` **is** touched, but only as a
  behaviour-preserving refactor onto the shared `review-ledger.md` (design D4) — the
  user relaxed the original don't-touch-`/audit` non-goal so the three skills share
  one source. Untangling reconciliation into its own skill / the audit-reconcile
  *writer* seam is **IMP-008**: different axis (new reconcile artefact + `slice
  reconcile` CLI), out of scope, not a sequencing prerequisite either direction.
- **Fork-invoked / parallel-raiser review** (IMP-024) and **RV verb e2e golden
  coverage** (IMP-029) — adjacent, separately owned. This slice rides the existing
  single-tree, parent-locus verb surface as-is.
- No new RV verbs or coordination mechanics. If `/inquisition` needs a new facet,
  that is a bounded enum + validation change, not a protocol change.

## Affected Surface

- `install/review-ledger.md` (**new** — shared protocol doc, auto-shipped to `.doctrine/`)
- `plugins/doctrine/skills/audit/SKILL.md` (refactor onto the shared doc)
- `plugins/doctrine/skills/code-review/SKILL.md` (**relocated** from `plugins/review/` + rewritten)
- `plugins/doctrine/skills/inquisition/SKILL.md`
- delete `plugins/review/` + drop its entry from `.claude-plugin/marketplace.json`
- `src/skills.rs` — **tests only** (code-review domain assertion `review→doctrine`,
  fixture); no production logic change. The embed/discover/`claude_links` suites
  stay green (behaviour-preservation gate).
- Skill re-embed path on any SKILL.md change (`doctrine claude install` + touch the
  embed crate) — see memory on skill refresh.
- **No facet enum change** — OQ-1 resolved to facet-by-target (design D2).

## Risks, Assumptions, Open Questions

All three OQs **resolved in `/design`** (see `design.md` §6/§7):

- **OQ-1 — inquisition facet.** RESOLVED → **facet-by-target**; posture carried by
  the `--raiser inquisitor` label (design D2). No enum change; inquisition is a
  conduct posture (ADR-009 axis), not a lifecycle facet.
- **OQ-2 — slice-less / diff-only targets.** RESOLVED → the **target ladder** (design
  D1): slice/phase → backlog item → create one → prose last-resort. RV `--target`
  stays a validated canonical ref; backlog kinds are valid targets, so "no slice"
  rarely means "no entity". Prose only for genuinely throwaway reviews.
- **OQ-3 — one slice or split.** RESOLVED → **one slice, zero production `src`**
  (design D5) — collapsed by OQ-1's no-enum answer.
- **ASM** — the existing `review` verb family is sufficient for all three skills'
  finding lifecycle (raise/dispose/verify/contest/withdraw); no new verb needed.

## Verification / Closure Intent

- All three SKILL.md files drive the `doctrine review` ledger end-to-end (open →
  raise → dispose/verify → synthesis) via the shared `review-ledger.md`, with their
  distinct voices intact. `/audit` observable behaviour is unchanged (refactor).
- The severity vocabulary each skill uses maps onto the RV `blocker|major|minor|nit`
  axis (and the close-gate teeth) coherently.
- No facet enum change (OQ-1 = facet-by-target). `src/skills.rs` test suite stays
  green with the code-review domain assertion updated `review→doctrine`.
- Skills re-embed and load; a smoke review (open an RV via each skill's flow,
  raise + dispose a finding) succeeds. `review` plugin removed from the marketplace
  with no dangling reference.
- IMP-023 backlog item updated to reflect the drift/reconciliation reassignment and
  closed when the two in-scope skills land.

## Follow-Ups

- **Minted by this slice's design** (to backlog at reconcile): cross-corpus
  **harvest-DRY** — `/audit`, `/notes`, `/handover`, `/next` (and now the review
  skills) all re-implement the "promote durable findings → notes/memory/backlog"
  step incompletely; extract a shared harvest contract. And **handover relocation**
  — fold the standalone `handover` plugin into doctrine core (plugin hygiene,
  orthogonal to the RV rewiring).
- Tracked separately: IMP-022 (drift ledger), IMP-008 (reconcile seam), IMP-024
  (fork/parallel-raiser), IMP-029 (RV verb e2e goldens).
