# Ship knowledge tiers (ADR-005)

## Context

ADR-005 (accepted) tiers shipped knowledge by access pattern: **PUSH** (the boot
snapshot, resident), **PULL-reference** (shipped `install/*.md` docs that skills /
boot point at), **skills** (thin routers). This slice realises that decision. The
ADR's resolutions (R-OQ-1…5, R-C1/C3/C5) and `inquisition.md` are the binding
spec — read them, not a paraphrase.

Two confessed defects motivate the work: `doc/glossary.md` is authoritative but
**unshipped** (embeds are `install/` `plugins/` `memory/` only), and a handful of
skills reproduce CLI flag syntax / storage-tier mechanics that drift when the CLI
moves (the C4-class failure origin).

## Scope & Objectives

Four legs, each bound to an ADR resolution. Altitudes kept apart: PUSH is a small
delta, PULL is two docs, skills shed only named offenders.

1. **Ship the glossary (R-OQ-1).** Relocate `doc/glossary.md` → `install/` so it
   enters the embed/ship set and lands in a client install. Resolve its
   `entity-model.md` cross-link at ship — inline the needed sentence or drop the
   link; **no dangling reference** into an unshipped build-spec. Wire a pointer so
   the doc is **reachable** (a skill / boot points at it — shipping ≠ visible).

2. **Author the CLI / hand-editing reference (R-C5).** A **new** shipped doc under
   `install/` (NOT `install/rules/AGENTS.md` — it lands at `.doctrine/rules/` where
   nothing reads it). Carries: *which verb for which intent*, hand-editing
   mechanics, storage-tier read/write discipline (read via `show`), edit-preserving
   rules. **Points to `doctrine --help`** for exact shapes; **never duplicates** it.
   Design must enumerate its **unique payload** vs `--help` / glossary / templates
   before authoring. Must be **pointed-at** by the skills that need it + boot.

3. **PUSH the reference-forms delta (R-OQ-5, R-C3).** Append a compact
   reference-forms block (entity-id pad rule, bare doc-local enums, the VT/VA/VH
   criteria modes) to `install/routing-process.md`. Storage-rule + use-CLI are
   **already resident** (commit 8206b67) — do not re-add. Keep the snapshot compact.

4. **De-dup named skill offenders (R-C1, evidence-bound).** Fix only the sites
   below; **no 20-skill sweep**. Each loses flag syntax / option tables / tier
   mechanics prose and gains a pointer to tier-1/2 + `--help`, per the ratified
   restate line (R-OQ-4): MAY name a verb / cite a rule by name; MUST NOT reproduce
   flag syntax, option/enum tables, or storage-tier mechanics prose.
   - `record-memory/SKILL.md:26-27,36-38,76` — command template + `--glob/--command/--tag` option table
   - `retrieve-memory/SKILL.md:27` — full flag template
   - `spec-product/SKILL.md:246-249` — command block + `--kind a|b` enums (already cites `--help` at :242)
   - `spec-tech/SKILL.md:13` — `--kind functional|quality` syntax
   - `execute/SKILL.md:27,47` + `phase-plan/SKILL.md:43` — repeated `slice phase … --status` lifecycle command (3×)
   - `canon/SKILL.md:25-27`, `inquisition/SKILL.md:39-40` — storage-tier mechanics **prose** (now PUSH-resident)

## Non-Goals

- **No 20-skill normalisation.** Untouched skills that already route (one-line
  storage-rule pointers in `slice:37`, `plan:40`) stay as-is.
- **No runtime-read loading model** (R-OQ-2 — keep compile-time embed; the
  rust-embed footgun is a known dev-loop cost, not a shipping defect).
- **No rename of `routing-process.md`** (R-OQ-3 — a rename churns every `@`-import
  + hook reference for cosmetic gain).
- **No new boot section** — the reference-forms block appends to the one existing
  Static asset (R-OQ-5).
- **OQ-00N → bare-OQ-N corpus normalisation** is a separate carry-in; out of scope
  here (Follow-Ups).
- No change to `doctrine --help` content or CLI command surface.

## Affected Surface

- `doc/glossary.md` → `install/glossary.md` (relocation; embed root). Drop its
  `entity-model.md` link (unshipped). **Ripple:** correct the `doc/glossary.md` path
  → `.doctrine/glossary.md` in 5 shipped templates (`spec-product/spec-tech/design/
  plan.md`, `plan.toml`) + `install/governance.md` (pre-existing client-dead link).
  Non-destructive breadcrumb in frozen SL-020 / spec-012 citations.
- `install/using-doctrine.md` — new operator doc (verbs, hand-editing, read-via-show).
- `install/routing-process.md` — append reference-forms block + reference-docs pointer.
- `src/install.rs` (`#[folder = "install/"]`) re-embeds new/moved files — the
  rust-embed recompile footgun applies (`cargo clean -p doctrine && cargo build`,
  then `doctrine boot`). Check `install/manifest.toml` if dir-create / gitignore
  wiring is needed for a new sub-path (likely not — flat `install/*.md`).
- `src/boot.rs` `boot_sequence` — assert the reference-forms block rides the
  snapshot (VT); confirm the new pull-reference docs are pointed-at.
- 6 skill sites above (de-dup) + any skill/boot text that becomes the pointer.

## Summary

Realises ADR-005's three tiers: ship + relocate the glossary, author one
pointed-at CLI/editing reference, push the reference-forms delta into the boot
digest, and shed CLI/tier duplication from 6 named skill sites. Bound to the ADR
resolutions; scope deliberately narrow (evidence-bound de-dup, delta-only PUSH).

## Risks / Assumptions / Open Questions

- **R-A1 (footgun).** A doc/asset edit is invisible until a full crate rebuild —
  `cargo clean -p doctrine && cargo build`, then `doctrine boot`. Build/run
  `./target/debug/doctrine`, not the stale PATH bin.
- **R-A2 (over-ship).** Relocating glossary must not drag unshipped build-specs;
  confirm the embed root sees only intended files.
- **R-A3 (entity-model link).** Decide inline-vs-drop in design; verify no dangling
  link survives in the shipped copy.
- **R-A4 (reachability).** A shipped-but-unreferenced doc is invisible (the
  AGENTS.md lesson). Every pull-reference must have a pointer — assert it.
- **Q1 (design).** Exact name + home of the CLI/editing reference doc, and its
  unique payload enumeration.
- **Q2 (design).** Where each pointer lives — in the skills, in boot, or both.

## Verification / Closure Intent

Per ADR-005 Verification:
- **VT** — glossary + CLI/editing reference present in a fresh client install
  (embed/ship test).
- **VT** — reference-forms block present in a regenerated boot snapshot.
- **VA** — no skill in the named set violates the restate line; pointers permitted.
- **VA** — no shipped doc reproduces `doctrine --help`; every pull-reference is
  pointed-at (reachability).
- Gate: `just check` green, `cargo clippy` zero warnings (bins/lib).

## Follow-Ups

- OQ-00N → bare-OQ-N corpus normalisation (separate slice/chore; may defer).
