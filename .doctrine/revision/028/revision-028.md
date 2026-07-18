# REV REV-028 — Semantic ownership and minimal projection

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-021 Claim C1 (semantic ownership + minimal projection), directionally accepted
at high confidence, is bound to a project-global principle by **ADR-019**
(embedding / publication / distribution / ownership / projection are five
independent asset properties; embedding ≠ projection). This REV lands the two
descending spec-prose amendments that principle requires. It does **not** touch
code — the minimal-projection *mechanism* descends later as implementation slices;
SPEC-009 becomes partly **forward-intent** (target architecture the shipped code
does not yet implement), which is a legal tech-spec posture provided *planned*
stays distinguishable from *verified*.

The library / publication / federated-search capability is a **separate contract
(B)** — a new PRD + tech spec — deliberately excluded here.

**Settled design forks (session, 2026-07-19):**
- ADR altitude (a): the separation principle is a new ADR (ADR-019), the shared
  root of both A and B — not buried in SPEC-009 prose.
- Base install shape: **three-file** base projection (`.gitignore`, `doctrine.toml`,
  `boot-project.md`). *Not* a fourth `governance.md` in the base set.
- Governance/orientation: standing governance is a **distinct, approval-gated,
  materialize-on-first-use surface**, never folded into `boot-project.md`. This
  reconciles minimal projection (nothing empty ships) with the mutation-authority
  rule (orientation is agent-improvable; governance is approval-gated) — the split
  is *drawn now* so a Stage-5 trust-acceptance mechanism inherits a lift-out.

Scope of the staged delta: **two entity `modify` rows** (SPEC-009, PRD-006) covering
decisions, structured `responsibilities` (both tiers), overview, and sources; plus a
**frozen requirement delta** — `create`/`modify`/`status` change rows for both specs,
authored into `revision-028.toml` *before* approval (F-2) and all landing `pending`
(F-3). This REV is the sanctioned amendment path, applied as a **surfaced-for-manual**
hand-edit at apply — not by hand-edit outside the REV.

**RV-285 rework (2026-07-19):** all eight findings accepted and folded in — canonical
seven-property vocabulary (F-1), frozen requirement rows (F-2), durable forward-intent
marker + `pending` standing (F-3), publication delivery confined to contract B (F-4),
structured responsibility edits (F-5), PRD-006 product requirements (F-6),
surface-split materialization with trust deferred to Stage 5 (F-7), phantom-`PHASE-05`
repair (F-8). Re-review of the frozen payload follows.

---

## Change row 1 — modify SPEC-009 (Install & distribution)

SPEC-009 today conflates embedding with projection ("the `install/` tree is
embedded via rust-embed and reproduced into a target directory"). The revision
splits them per ADR-019, introduces minimal base projection + materialize-on-demand,
seeds the semantic-source-root direction, and repairs a stale anchor. Container C4
level, `descends_from` PRD-006, `parent` SPEC-003 are unchanged.

**Edit 1.0 — durable forward-intent marker (F-3).** After apply, `spec show
SPEC-009` is the evergreen surface; the temporary REV rationale is not visible there.
So a durable current-state/target-state banner is staged into the `## Overview`:

> **Status — partly forward-intent (RFC-021 C1 / ADR-019).** The embed-as-storage,
> root-detection, idempotent-plan, and `asset_text` mechanisms are **shipped and
> verified**. The projection model described below — minimal three-file base,
> embed/publish/project separation, semantic source roots, materialize-on-demand
> surfaces (D6–D9) — is **target architecture**: authored intent the shipped code
> does not yet implement. Its requirements stand `pending` until an implementation
> slice lands them and reconciliation verifies coverage. Do not read D6–D9 or the
> `pending` requirements as current behaviour.

Correspondingly, **every requirement the delta below revises or adds is set to
`pending`** (planned), not `active`: the revised statements (REQ-164/165/171)
describe target behaviour the current code no longer satisfies, so their standing
transitions `active → pending` via `status` change rows; the new requirements are
created `pending`. They return to verified only through implementation-slice
reconciliation, never inferred from the spec.

**Edit 1.1 — Decision D1 (distribution is a compile-time embed).**

- *Before:* "**D1 — distribution is a compile-time embed, not a bundle.** The
  `install/` tree is embedded via rust-embed and reproduced at runtime; there is no
  second asset artefact and no network fetch. The manifest is embedded but excluded
  from the installed fileset — it configures the install rather than being installed."
- *After:* "**D1 — embedding is a storage mechanism, not the projection policy
  (ADR-019).** Compile-time rust-embed remains the production *storage* default —
  one self-contained binary, no sidecar bundle, no network fetch. It no longer
  defines the *installed* fileset: what lands in a project is drawn from an explicit
  projection policy (D7), not from what the binary happens to embed. The manifest is
  embedded but excluded from every projected set."

**Edit 1.2 — Decision D4 (distributability declared in the manifest).**

- *Before:* "**D4 — distributability is declared in the manifest.** A new authored
  kind is made distributable by adding its directory to `[dirs].create` and negating
  its derived/runtime subtrees narrowly in `[gitignore].entries` — never by changing
  install code."
- *After:* "**D4 — the manifest declares projection *policy*, not embed contents.**
  The manifest declares the minimal base fileset (D7), the materialize-on-demand
  surfaces (D8), the dirs, gitignore negations, and root markers. A new authored kind
  is made *materializable* — its root appears on first use — via a manifest edit, not
  by base-projecting its tree or changing install code."

**Edit 1.3 — new Decisions D6–D9 (append to Decisions).**

- **D6 — the seven independent asset properties (ADR-019).** Cite ADR-019's
  canonical vocabulary *unchanged* — owned, versioned, distributed, embedded,
  runtime-loaded, published, projected — no property implies another; in particular
  embedding ≠ publication ≠ projection. (Publication *policy* only; publication
  *delivery* is contract B.)
- **D7 — minimal, justified base projection.** The base install is exactly three
  files — `.gitignore`, `doctrine.toml`, `boot-project.md`. A file is projected only
  when its presence at a stable project path is operationally necessary or materially
  improves discoverability, integration, control, or supported customization. No
  template, reference doc, agent definition, hymn, or integration asset is projected
  by default.
- **D8 — materialize on first use.** Surfaces that are distinct but not yet populated
  are not shipped empty: entity roots, customized hymns, and standing governance
  appear on disk only when first used/customized — the pattern already used for entity
  roots and hymns.
- **D9 — governance is a *physically distinct* surface (trust deferred).** Standing
  project governance materializes on **explicit user definition** into its own
  surface (candidate `governance.md`), never folded into the agent-improvable
  `boot-project.md` orientation surface. This draws the volatility /
  mutation-authority boundary up front (ADR-019 position 3). Enforcing trust/approval
  on edits to that surface is a **Stage-5 concern, explicitly out of scope here** —
  D9 establishes physical separability, not an approval mechanism.

**Edit 1.4 — Responsibilities (STRUCTURED `responsibilities` TOML, mirrored in prose).**
This is a **structured** edit to `spec-009.toml`'s `responsibilities = [...]` array
*and* its `## Responsibilities` prose mirror — applying prose alone would leave the
synthesized `spec show` asserting the old embed-and-lay-down contract (F-5). Landed
at apply through the sanctioned manual edit of both tiers, then the post-apply
synthesized view is verified.

- *Before, entry 1:* "Embed the `install/` source tree into the binary at compile
  time (rust-embed `#[folder]`) and reproduce it into a target directory under a
  user's project, so a single self-contained binary carries everything a fresh
  project needs — no network, no separate asset bundle."
- *After, entry 1 (split into two):*
  - "Embed the semantic source roots into the binary at compile time (rust-embed) as
    the **storage** mechanism — one self-contained binary, no network, no sidecar
    bundle. Embedding is storage only; it does not define the projected fileset."
  - "Project a **minimal, explicitly justified** base fileset from the manifest's
    projection policy (not from the embed contents), and materialize non-base
    surfaces — entity roots, customized hymns, standing governance — on first use."
- *Entries 2–6 (manifest, root detection, plan, `asset_text`, wiring contract):*
  retained, with entry 2 reworded so the manifest owns the **projection policy**
  (base fileset + materialize-on-demand surfaces), not just dirs/gitignore/markers.
- The `## Overview` prose is reframed to match: embed the semantic source roots as
  storage; project a minimal base; materialize the rest. Physical source roots follow
  semantic ownership (directional per ADR-019: templates / guidance / sealed
  definitions / reference / integrations / base-projection policy / memory corpus)
  rather than one `install/` root — exact names deferred to the implementing slice.

**Edit 1.5 — anchor repair (sources).** Drop the stale `markdown doc/install-spec.md`
`[[source]]` (the file no longer exists). Live anchors `src/install.rs`,
`install/manifest.toml`, `src/root.rs` are retained. No new source root is anchored
yet (forward-intent: the roots do not exist until the implementing slice creates them).

**Edit 1.6 — repair the phantom `PHASE-05` interaction (F-8).** The `## Overview`
currently says the skills-sibling interaction edge "is `PHASE-05`". No slice is
named and `interactions.toml` is empty — a mobile phase label is neither a durable
relation nor a resolvable anchor. Delete the `PHASE-05` claim; state the
skills-distribution sibling boundary (PRD-003 / SPEC-010) in prose without citing a
phantom phase. Authoring a durable `spec interactions` edge to SPEC-010 is left to
the implementing slice (it is a peer `uses` edge, not containment).

### SPEC-009 requirement delta (frozen as change rows; all land `pending`)

Every row below is authored as a REV-028 `[[change]]` row *before* approval (F-2) and
lands its requirement at `pending` (F-3). Each materialization obligation is a
**separate, independently testable** requirement with its own trigger and actor
(F-7) — no compound "materialize everything on first use" requirement.

*Revised (statement changes → `active → pending` status row + modify row):*
- **REVISE FR-001 (REQ-164):** "embed the semantic source roots at compile time as the
  *storage* mechanism" — projection removed from this requirement.
- **REVISE FR-002 (REQ-165):** "the manifest declares the **projection policy** (base
  fileset + materialize-on-demand surfaces + dirs + gitignore + markers), embedded and
  excluded from every projected set."
- **REVISE NF-002 (REQ-171):** "distribution is one self-contained binary with a
  compile-time embed as the *storage* mechanism — no network fetch, no sidecar
  bundle" (embed reframed as storage, not the installed set).

*New (created `pending`):*
- **ADD FR — minimal base projection:** project exactly the three-file base
  (`.gitignore`, `doctrine.toml`, `boot-project.md`) from the declared projection
  policy, never from embed contents. Trigger: `doctrine install`. Actor: installer.
- **ADD FR — entity roots materialize on first use:** an authored-kind root is created
  on first use of that kind, not at install. Trigger: first scaffold of the kind.
  Actor: the entity-scaffolding verb.
- **ADD FR — customized hymns materialize on customization:** a hymn appears on disk
  only when first customized. Trigger: explicit hymn customization. Actor: the
  customization verb.
- **ADD FR — standing governance materializes on explicit definition:** the governance
  surface (candidate `governance.md`) is created only when the user explicitly defines
  standing governance, as a surface **physically distinct** from `boot-project.md`.
  Trigger: explicit user governance-definition command. Actor: the user via that
  command. **Trust/approval enforcement on edits is out of scope (Stage 5).**
- **ADD NF — no default auxiliary projection:** no template, reference doc, agent
  definition, hymn, or integration asset is projected into a client repo by default.
- **ADD NF — governance/orientation surface distinctness:** standing governance never
  ships as a base projection and is never folded into the `boot-project.md`
  orientation surface.
- **UNCHANGED (stay `active`):** FR-003 (root detection), FR-004 (inspectable plan),
  FR-005 (`asset_text` seam), FR-006 (kind wiring), NF-001 (idempotent
  never-overwrite), NF-003 (skills/memory not absorbed).

---

## Change row 2 — modify PRD-006 (Install)

The product intent shifts from "provision the files Doctrine ships" to "provision a
**minimal, justified** base, and publish/materialize the rest." This is a genuine
product-intent change (what the operator's repo ends up containing), not just
mechanism, and it improves fresh-install ergonomics — a clean, legible `.doctrine/`.

**Edit 2.1 — §1 Intent (append).** Add: provisioning is *minimal and justified* —
only files with an operational, discoverability, integration, control, or
customization reason land at a stable project path; other framework-owned material is
published (inspectable/copyable) or materialized on demand, not installed by default.
User control is served by a publication surface, not by projecting files no one
services.

**Edit 2.2 — §2 Scope.**

- *Before (in-scope):* "Provisioning Doctrine's working files into a target project —
  creating the directories it needs and writing the files it ships."
- *After (in-scope):* "Provisioning a **minimal, explicitly justified** base fileset
  into a target project; creating the directories it needs; materializing further
  surfaces (entity roots, standing governance) on first use / explicit definition."
- *After (out-of-scope, add):* "**Publishing** framework-owned material for
  inspection/copy — that is a sibling capability (contract B: library / publication /
  search), a dependency of the user-control story but **not** part of install's
  scope. Install neither publishes nor depends on publication being delivered."

This removes the earlier in-scope publication promise (F-4): A's success is minimal
projection, not delivery of B.

**Edit 2.3 — Responsibilities (STRUCTURED, F-5).** PRD-006 has **no prose
Responsibilities section** — its `responsibilities` live only in `spec-006.toml`. So
this is a structured TOML edit, not a prose one:
- *Revise* the establish-files responsibility → "Establish a **minimal, justified**
  base of Doctrine's working files inside a project, materializing further surfaces on
  demand — a home to operate from without projecting material no one services."
- *Add* "Keep the projected footprint minimal — only files with a stated project-level
  reason occupy a stable path; standing governance and volatile orientation stay
  physically distinct surfaces."

**Edit 2.4 — PRD-006 requirement delta (F-6; frozen rows, land `pending`).** The
product intent must be enforced by a *product* requirement, or all-files projection
would still satisfy PRD-006 after the prose change. Altitude split: the PRD carries
the **intent** as one requirement; its **testable decomposition** lives in SPEC-009
(FR-007 minimal base, NF-004 no default aux, NF-005 governance distinctness, FR-008–010
materialization) which descend from PRD-006. So the product-tier change is a single
comprehensive revision, not granular product requirements.

- **REVISE FR-001 (REQ-043):** from "Provision Doctrine's files into a project" →
  "Provision a **minimal, explicitly justified** base fileset into a project —
  auxiliary framework material (templates, references, agent definitions, hymns,
  integrations) is **not** projected by default, and further surfaces (entity roots,
  standing governance) materialize on demand." (`active → pending`; rows frozen above.)
- **Mechanism note:** the REV `introduce` change row is **SPEC-only** (`--member-of`
  refuses a PRD), so new *product* requirements cannot be declared as REV rows. This
  reinforces the altitude split — the testable obligations are SPEC-009's (frozen as
  `introduce` rows above); the PRD change is the REQ-043 revision. (Candidate tooling
  gap noted for backlog: no REV path to introduce a product requirement.)
- **UNCHANGED:** FR-002 (REQ-044, preview plan), FR-003 (REQ-045, resolve root),
  FR-004 (REQ-046, ignore boundary), NF-001 (REQ-047, converge), NF-002 (REQ-048,
  self-contained) — NF-002's "shipped defaults" still hold (embed remains the storage
  source of the base fileset).

Out-of-scope otherwise unchanged (meaning of provisioned files, upgrade/migrate,
de-provision, VCS choice).
