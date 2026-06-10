# Standard (STD) governance kind

## Context

`standard` (`STD-NNN`) is the third planned governance kind in `glossary.md`,
grouped with ADR and `policy` (`POL`). SL-030 introduced POL by extracting a
shared spine — `src/governance.rs` — that the kind-blind entity engine is
parameterized over via a `GovKind` descriptor (`kind`, `stem`, `statuses`,
`hidden`). `adr.rs` and `policy.rs` are now thin **data** modules binding their
`*_KIND` constant into that spine. SL-030's design (§4 "share the mechanism,
keep the identity") and §6 explicitly anticipated STD as the next rider: "STD is
imminent."

STD records *standing rules* (like POL), distinct from ADR's *decisions*. The
slice rides the existing spine — no new mechanism — and closes one debt SL-030
left in `src/boot.rs`: the per-kind boot projection is not yet parameterized.

## Scope & Objectives

- **`src/standard.rs`** — thin data module mirroring `src/policy.rs`: `STD_KIND:
  GovKind`, a concrete clap `StandardStatus` enum, `STANDARD_STATUSES` known-set,
  `is_hidden` hide-set, and CLI forwarders (`run_new`/`run_list`/`run_show`/
  `run_status`) binding `STD_KIND` into `governance::*`. Drift canary +
  hide-set-⊆-known-set tests mirror POL's.
- **Scaffold templates** — `install/templates/standard.{toml,md}` (rust-embedded).
  TOML mirrors `policy.toml` (metadata + inert `[relationships]`, `status =
  "draft"`); MD body reuses tuned prior art if a standard template exists under
  `../spec-driver/supekku/templates/` (attributed), else mirrors `policy.md`
  sections — **no YAML frontmatter** (storage rule / ADR-D1: metadata in the
  sister TOML).
- **CLI wiring** — `doctrine standard new|list|show|status` registered in the
  command tier, mirroring the `policy` subcommand registration.
- **Install wiring** — manifest dir + `.gitignore` negation for the new authored
  entity tree `.doctrine/standard/` (the authored-entity-wiring pattern).
- **Action 2 (boot parameterization)** — collapse `boot.rs`'s per-kind governance
  projection. Today `SourceKind` carries one variant per kind (`Adrs`, `Policies`)
  and `produce` has one near-verbatim match arm each, differing only in the
  `GovKind` and the status-filter set. Replace both with a single data-carrying
  variant (kind + in-force status **set**) and one arm, so STD's boot surface —
  and any future governance kind — is a **one-line `boot_sequence()` addition**,
  not a new variant + new arm. The filter is a `&'static [&'static str]` (not a
  single literal): STD has two in-force statuses (`default`, `required`), and
  `ListArgs.status` is already a `Vec`, so the set costs nothing. The new variant
  is **not** named `Governance` (that name already binds the `governance.md` disk
  reader) — e.g. `GovRows(&'static GovKind, &'static [&'static str])`. The ADR and
  POL section bytes must stay byte-identical (behaviour-preservation gate); the
  existing boot suites are the proof.
- **Glossary** — STD row already present (`STD-123`); no change intended
  (parity with SL-030 OQ-1).

## Non-Goals

- **No new spine mechanism.** STD reuses `governance.rs` as-is. If STD exposes a
  need the spine cannot meet, that is a `/consult`, not silent spine surgery.
- **Inherited, shared gaps stay deferred** (SL-030 §5.5/§6): boot's error≡empty
  marker collapse, supersession⇏status, the inert `--tag` axis. STD inherits POL's
  parity on all three; none are fixed here.
- **No `standard supersede` verb** — parity with the unbuilt `adr`/`policy`
  supersede (SL-030 §6 follow-up).
- Not an ADR (no project-global decision) and not evergreen `doc/*` spec — this is
  one shippable capability slice.

## Summary

Third governance kind by the established POL playbook (thin data module over the
shared spine) plus the boot-projection parameterization SL-030 deferred.

## Follow-Ups

- **RESOLVED (design):** STD status vocabulary = `draft / default / required /
  deprecated / retired`. STD is a *sibling* of POL (both standing rules) that
  gains a `default` tier — "recommended unless justified to deviate" (supekku
  prior art), distinct from `required` (mandatory). In-force (boot) = the **set**
  `{default, required}`; hide-set = `{deprecated, retired}`; the template seeds
  `draft`. This is why the boot parameterization carries a status set, not a
  single literal (see Action 2).
- After this slice, the spine carries three riders (ADR/POL/STD) and boot is fully
  data-driven — a fourth kind would be pure data, no code-shape change. Worth a
  durable memory at close.
