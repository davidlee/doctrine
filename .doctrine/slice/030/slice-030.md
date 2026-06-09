# Policy entity kind (POL)

## Context

`policy` (`POL-123`) is a planned governance kind in `glossary.md` — grouped with
`standard` (`STD`) and `architecture decision record` (`ADR`) — but it has no
representation: no CLI verb, no entity tree, no boot-snapshot surface. ADR is the
only governance kind that has shipped; it rides `entity::Kind` as a top-level
reserved kind over the kind-blind engine (`src/adr.rs`, slice SL-006), with a
sister `adr-NNN.toml` + scaffolded `adr-NNN.md` + `NNN-slug` symlink.

`entity-model.md` § Adjudication leaves the governance shape open: "`decision` /
governance → policy, standard, ADR → one kind + `doc_kind`, **or ADR separate if
decision semantics earn it**." ADR shipped separate. Whether POL is its own
reserved kind (ADR-shaped) or a `doc_kind` facet on a shared governance entity is
the central design question this slice must adjudicate before any code.

An ADR records a *decision*; a policy states a *standing rule* — different
lifecycle (a policy is amended/retired, not "superseded by" a later decision),
different status vocabulary. That semantic gap is the evidence the design weighs.

## Scope & Objectives

- **Introduce the `policy` authored entity** so `doctrine policy new|list|show`
  (and a status transition) scaffold and query `POL-NNN` entities, honouring the
  storage rule (structured data in `policy-NNN.toml`, prose in `policy-NNN.md`).
  Ride the existing entity engine — no parallel implementation.
- **Wire the three install surfaces** a new authored type requires
  (`mem.pattern.install.authored-entity-wiring`): `install/manifest.toml`
  `[dirs].create`, the `.gitignore` `!.doctrine/policy/` negation (else the tree
  is silently uncommittable), and parity with `slice`/`adr`/`spec`.
- **Surface accepted/active policies in the boot governance snapshot** — a new
  `SourceKind` + section in `src/boot.rs`, rendered alongside `Accepted ADRs`, so
  agents pay for policy governance once per change, not once per session.

## Non-Goals

- **`standard` (`STD`).** A sibling governance kind; same adjudication may apply,
  but it is out of scope here. If the design lands a generalised governance
  substrate, STD becomes a trivial follow-up — captured, not built.
- **Policy enforcement / linting.** This slice ships the *entity*, not any
  machinery that checks code or process against a policy.
- **Relations to specs/requirements/slices** beyond what the scaffold already
  reserves. No coverage gates.
- **Retrofitting existing prose rules** (CLAUDE.md, conventions) into `POL`
  entities — a content-migration question for later.

## Summary

Ship `policy` as a queryable governance entity with full CLI + install wiring +
boot-snapshot projection, reusing the ADR seam verbatim where the semantics match
and diverging only where policy lifecycle genuinely differs. The own-kind vs
`doc_kind`-facet decision is deferred to `/design`.

## Follow-Ups

- `standard` (`STD`) entity kind — sibling, deferred (a third thin kind on the
  shared `governance.rs` spine this slice introduces).
- **Governance tag reader** — `--tag` is inert for governance kinds (ADR's
  `key()` returns no tags; `Meta` never reads them). Extend the metadata reader so
  tag filtering works for ADR/POL/STD.
- **`policy supersede` verb** — mechanically flip a superseded policy off
  `required` (parity with ADR's unbuilt `adr supersede`/F1); enforces the
  design's supersession invariant.
- **boot error vs empty** — `section_or_marker` collapses producer errors and
  emptiness into one marker, hiding corruption; boot-wide disambiguation.
