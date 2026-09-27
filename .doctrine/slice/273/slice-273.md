# Library ownership and lib: citations

## Context

RFC-033 (Learning surface: corpus unification and format) settled that the
published reference library — `install/*.md`, read in clients as
`reference/<name>.md` through `doctrine library show` — is the **normative
owner** of doctrine's rules and concepts. REV-067 wrote that into governance:

- **ADR-005** now names an owner per material type, adds memories as tier 4,
  and carries two new invariants: a memory cites library-owned material and
  never restates it; a library citation carries the `lib:` marker and must
  resolve.
- **ADR-024** now marks its published logical address with `lib:` —
  `lib:reference/glossary.md` — where stripping the prefix yields the argument
  `doctrine library show` takes.

Neither change has reached the corpus. The library was never audited as a
system (SL-144, now abandoned, scoped that audit in June, before ADR-019), and
every existing library citation uses the bare form `<name>.md` that boot
currently teaches. A first grep counts ~120 bare `*.md` citations across
skills, templates and the routing digest; the number is an undercount until
the sweep regex carries the full prefix set.

This is RFC-033's slice S1. It fulfils CHR-023 (ADR-005 compliance), which
SL-144 was meant to fulfil.

## Scope & Objectives

1. **Library ownership audit and consolidation.** Audit the 17 `install/*.md`
   documents as one information architecture: which rule or concept each owns,
   where they overlap, where they conflict, what is missing. Consolidate so
   each rule or concept has exactly one owning doc and the others cite it.
   Known overlaps: `routing-process.md` (boot digest *and* published doc),
   `using-doctrine.md`, `glossary.md`, `authority.md` / `authority-model.md`.
2. **Introduce the `lib:` form.** Boot and `shipped-corpus-authoring.md` teach
   `lib:<address>` in place of "cited bare as `<name>.md`" — once, in the boot
   source that owns the reference-docs paragraph.
3. **Citation sweep.** Rewrite every library citation in the shipped non-memory
   corpus — skills, templates, hymns, published docs, boot sources — to the
   `lib:` form. Classify before rewriting: a bare `<name>.md` that names a
   *client file* (e.g. the project's own `.doctrine/governance.md`) is not a
   library citation and stays.
4. **Restate-line audit** (ADR-005's R-OQ-4, carried from SL-144): skills
   reproducing flag syntax, option/enum tables, or storage-tier mechanics as
   prose are cut to pointers.

**Execution note (user, 2026-09-27):** the citation sweep (objective 3) runs
on a pi worker using DeepSeek — looser than Sonnet, but cheap and fast enough
that a second pass, if needed, still comes out ahead. The sweep therefore
needs a mechanical exit criterion (see OQ-1).

## Affected surface

- `install/*.md` — the reference library (objectives 1, 2, 3)
- `install/templates/**`, `install/hymns/**` — citations (objective 3)
- `plugins/doctrine/skills/**` — citations and restate line (objectives 3, 4)
- `src/boot.rs` and any boot source strings carrying the reference-docs
  paragraph (objective 2)
- `publication/manifest.toml` — only if consolidation retires or renames a
  published doc

## Non-Goals

- **Memories.** Rewriting shipped memories into pointers and sweeping their
  citations is S2.
- **Human documentation** (RFC-017's set) — S3.
- **SL-242's ground:** untracking the projection residue, re-anchoring the four
  stale memories, settling the manifest pointer. SL-242 proceeds
  independently; the overlap is its doctor check (OQ-2).
- **Project override of library rules** — QUE-228.
- **Client migration** of existing installs — out of scope by RFC-033's
  already-rejected list.
- Editing projected skill copies (`.agents/skills/`, `.claude/skills/`); only
  `plugins/doctrine/skills/` masters are authored.

## Risks & assumptions

- **R1 — Sweep false positives.** Bare `<name>.md` also names client files and
  repo-local paths. A mechanical rewrite without classification would mint
  `lib:` citations that do not resolve. Mitigation: the resolution check
  (OQ-1) fails every such mint.
- **R2 — Consolidation churn under the sweep.** Retiring or merging a library
  doc changes citation targets. Mitigation: consolidate first, sweep second.
- **R3 — Looser worker.** DeepSeek may miss or over-rewrite. Mitigation: the
  mechanical check plus a diff review at import; re-run is cheap.
- **A1 — Library addresses are stable** except where consolidation
  deliberately changes them.

## Open questions

- **OQ-1** — Does the `lib:` resolution check land here, as the sweep's exit
  criterion, rather than in S4? Recommendation: yes, a scan limited to the
  surfaces this slice touches; S4 widens it to memories and the human set.
- **OQ-2** — SL-242's objective 4 proposes a doctor check for bare `<name>.md`
  citations with no published counterpart. With `lib:`, that check is the same
  check. Does this slice absorb it (and SL-242 drop objective 4), or sequence
  behind SL-242?
- **OQ-3** — One slice or two? The audit/consolidation (judgement, Sonnet or
  better) and the sweep (mechanical, DeepSeek) have different executors. They
  are kept together here because R2 orders them; split if the design shows
  they can ship separately.

## Verification / closure intent

- Each rule or concept in the library has one owning doc; the audit artefact
  records the owner map (VA).
- Boot and `shipped-corpus-authoring.md` teach `lib:` (VT: boot assertion).
- No bare library citation survives in the swept surfaces, and every `lib:`
  citation resolves to a publication-register entry (VT: the check from OQ-1).
- No skill violates the restate line (VA).
- `doctrine check gate` green at close.

## Summary

Make the reference library the owner it is now declared to be, and make every
citation of it recognisable and checkable.

## Follow-Ups

- S2 (memories as curriculum), S3 (human set), S4 (wider enforcement) —
  RFC-033.
