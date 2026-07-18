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

Scope of the staged delta: **two `modify` prose rows** (SPEC-009 entity, PRD-006
entity), plus a **requirement delta** (below) whose frozen `create`/`modify` change
rows are added after the bundle review settles the requirement set. Both prose
targets amend authored spec truth; this REV is the sanctioned amendment path,
applied as a **surfaced-for-manual** hand-edit at apply — not by hand-edit outside
the REV.

---

## Change row 1 — modify SPEC-009 (Install & distribution)

SPEC-009 today conflates embedding with projection ("the `install/` tree is
embedded via rust-embed and reproduced into a target directory"). The revision
splits them per ADR-019, introduces minimal base projection + materialize-on-demand,
seeds the semantic-source-root direction, and repairs a stale anchor. Container C4
level, `descends_from` PRD-006, `parent` SPEC-003 are unchanged.

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

- **D6 — five independent asset properties (ADR-019).** Owned, versioned,
  distributed, embedded, and project-projected are independent; no property implies
  another. Publication is distinct from projection: framework-owned material may be
  inspectable and copyable through a declared seam without being installed.
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
- **D9 — governance is a distinct, approval-gated surface.** Standing project
  governance materializes on first use into its own surface (candidate
  `governance.md`), never folded into the agent-improvable `boot-project.md`
  orientation surface. This draws the volatility / mutation-authority / trust boundary
  up front (ADR-019 position 3).

**Edit 1.4 — Overview + Responsibilities reframe.** The Overview's "embeds the whole
`install/` source tree … and reproduces it into a target directory" and the mirrored
responsibility become: embed the **semantic source roots** as compile-time storage;
**project a minimal base fileset** from an explicit projection policy; **materialize**
non-base surfaces on first use. Physical source roots follow semantic ownership
(directional per ADR-019: templates / guidance / sealed definitions / reference /
integrations / base-projection policy / memory corpus) rather than one `install/`
root — exact names deferred to the implementing slice.

**Edit 1.5 — anchor repair (sources).** Drop the stale `markdown doc/install-spec.md`
`[[source]]` (the file no longer exists). Live anchors `src/install.rs`,
`install/manifest.toml`, `src/root.rs` are retained. No new source root is anchored
yet (forward-intent: the roots do not exist until the implementing slice creates them).

### Requirement delta (change rows added post-review)

- **REVISE FR-001 (REQ-164):** from "embed the install source tree … and reproduce it
  into a target directory" → "embed the semantic source roots at compile time as the
  *storage* mechanism" (projection removed from this requirement).
- **REVISE FR-002 (REQ-165):** manifest parsed for **projection policy** (base fileset
  + materialize-on-demand surfaces + dirs + gitignore + markers), embedded and excluded.
- **REVISE NF-002 (REQ-171):** "one self-contained binary, compile-time embed as the
  *storage* mechanism, no network fetch, no sidecar bundle" (embed reframed as storage,
  not the installed set).
- **ADD FR — minimal base projection:** project exactly the three-file base
  (`.gitignore`, `doctrine.toml`, `boot-project.md`) from the declared projection
  policy, not the embed contents.
- **ADD FR — materialize on first use:** create entity roots, customized hymns, and
  standing governance on first use, not at install.
- **ADD NF — no default auxiliary projection:** no template, reference doc, agent
  definition, hymn, or integration asset is projected by default.
- **ADD NF — governance surface distinctness:** standing governance never ships as
  base projection and is a distinct surface from `boot-project.md` orientation.
- **UNCHANGED:** FR-003 (root detection), FR-004 (inspectable plan), FR-005
  (`asset_text` seam), FR-006 (kind wiring), NF-001 (idempotent never-overwrite),
  NF-003 (skills/memory not absorbed).

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
  surfaces (entity roots, standing governance) on first use. Publishing the remaining
  framework-owned material for inspection/copy without projecting it (the library
  surface is a sibling capability, contract B)."

**Edit 2.3 — Responsibilities (add one).** "Keep the projected footprint minimal —
only files with a stated project-level reason occupy a stable path; standing
governance and volatile orientation stay distinct surfaces."

Out-of-scope is unchanged (meaning of provisioned files, upgrade/migrate, de-provision,
VCS choice).
