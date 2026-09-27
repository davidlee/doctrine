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

Shaped by DEC-339 … DEC-346 (inquiry, 2026-09-27).

1. **Library ownership audit and consolidation.** Audit the 17 `install/*.md`
   documents as one information architecture: which rule or concept each owns,
   where they overlap, where they conflict, what is missing. Consolidate so
   each rule or concept has exactly one owning doc and the others cite it.
   - `routing-process.md` is recast as the boot's compact onboarding summary
     and renamed (DEC-344): it owns routing table, postures, core process and
     guardrails; for `glossary.md` / `using-doctrine.md` essentials it carries
     per-message essentials only, each ending with a `lib:` cue to its owner.
     ADR-005's text changes through a Revision.
   - The first-use qualification rule (C5) moves to `glossary.md`.
   - `boot-footer.md` is retired — manifest entry and asset (DEC-343).
   - `using-doctrine.md` gains the publication model and `lib:` form section
     (G7, DEC-342). The other skill-only gaps are IMP-500.
2. **Introduce the `lib:` form.** The boot onboarding summary carries the
   resolution rule — a `lib:<address>` citation is read with `doctrine library
   show`, not on disk — and states that a retrieval a skill or reference doc
   specifies is mandatory (DEC-342, DEC-345). `shipped-corpus-authoring.md`
   teaches the author side, citing `using-doctrine.md` (DEC-343).
   `doctrine library show` accepts the `lib:` prefix verbatim.
3. **Citation resolution check** (DEC-339; absorbed from SL-242, DEC-340). One
   pure scanner plus the publication resolver, two callers: a build-repo test
   over shipped surfaces (the sweep's finish line) and a `doctor` leg over
   client `.doctrine/**`. Fails any unresolved `lib:` address; reports bare
   occurrences of library docs (warning until the sweep lands, then failing).
4. **Citation sweep** (DEC-341). Rewrite every library citation in the shipped
   non-memory corpus — skills, templates, hymns, published docs, boot sources —
   to the `lib:` form, from an adjudicated inventory: a worker records every
   occurrence with a recommended disposition, the orchestrator adjudicates,
   the worker implements only the adjudicated rows, the orchestrator verifies.
   Colliding library filenames are not renamed.
5. **Restate-line audit** (ADR-005's R-OQ-4; DEC-345). Flag and option shapes
   cut to `--help` pointers; concepts with a library owner cut to `lib:`
   citations; ownerless concept tables stay, their locations logged in IMP-500.
   Shares the sweep's inventory.

**Execution (DEC-346).** Four phases: (1) resolver + check, Claude, TDD;
(2) teaching + consolidation incl. the ADR-005 Revision and rename, Claude;
(3) one combined inventory (citations + restate offenders) by a DeepSeek pi
worker, adjudicated by the orchestrator; (4) DeepSeek implements the
adjudicated inventory, the orchestrator verifies and flips the bare-occurrence
report to failing. Audit adds a second combined DeepSeek pass to catch
anything that slipped through.

## Affected surface

- `install/*.md` — the reference library (objectives 1, 2, 4)
- `install/templates/**`, `install/hymns/**` — citations (objective 4)
- `plugins/doctrine/skills/**` — citations and restate line (objectives 4, 5)
- `src/boot.rs` — the renamed boot embed (objectives 1, 2)
- the library-show command, the publication resolver, `src/doctor_checks.rs`,
  and a build-repo test (objectives 2, 3)
- `publication/manifest.toml` — the rename and `boot-footer.md` retirement
- ADR-005 — via Revision (objective 1)
- IMP-500 body — ownerless-table locations (objective 5)
- `.doctrine/slice/273/` — the tracked sweep inventory (objectives 4, 5)

## Non-Goals

- **Memories.** Rewriting shipped memories into pointers and sweeping their
  citations is S2.
- **Human documentation** (RFC-017's set) — S3.
- **SL-242's ground:** untracking the projection residue, re-anchoring the four
  stale memories, settling the manifest pointer. SL-242 was amended to cede its
  doctor check here (DEC-340).
- **Project override of library rules**, including what `governance.md` should
  be — QUE-228 (DEC-343).
- **Library owners for G1–G6, G8** — IMP-500 (DEC-342).
- **Deterministic inlining of `lib:` references into skills** — IDE-060.
- **Client migration** of existing installs — out of scope by RFC-033's
  already-rejected list.
- Editing projected skill copies (`.agents/skills/`, `.claude/skills/`); only
  `plugins/doctrine/skills/` masters are authored.

## Risks & assumptions

- **R1 — Sweep false positives.** Bare `<name>.md` also names slice artefacts,
  client files and repo-local paths. Mitigation: nothing is rewritten without
  an adjudicated inventory row (DEC-341); the check fails any unresolved mint.
- **R2 — Consolidation churn under the sweep.** The rename and retirements
  change citation targets. Mitigation: consolidate first, sweep second
  (DEC-346).
- **R3 — Looser worker.** DeepSeek may miss or over-rewrite. Mitigation:
  adjudicated inventory, the check, verification against the inventory, and
  an audit re-pass.
- **R4 — Skipped retrieval.** Guidance cut from a skill to a citation costs a
  tool call an agent will sometimes skip. Mitigation: the boot mandate and
  per-message essentials kept in the boot summary (DEC-345); structural fix
  deferred to IDE-060.
- **R5 — Embed staleness.** Edits under `install/` are invisible to boot and
  `library show` until rebuilt; verification runs after a rebuild.
- **A1 — Library addresses are stable** except where consolidation
  deliberately changes them (the rename, `boot-footer.md`).

## Open questions

Settled in inquiry: OQ-1 → DEC-339; OQ-2 → DEC-340; OQ-3 (one slice or two) →
one slice, ordered by DEC-346.

## Verification / closure intent

- Each rule or concept in the library has one owning doc; the audit artefact
  records the owner map (VA).
- The boot summary teaches `lib:` and the mandatory-retrieval rule;
  `shipped-corpus-authoring.md` teaches `lib:` (VT: boot assertion).
- `doctrine library show lib:<address>` resolves (VT).
- No bare library citation survives in the swept surfaces, and every `lib:`
  citation resolves to a publication-register entry (VT: the build-repo test).
- The doctor leg reports an unresolved `lib:` citation in client
  `.doctrine/**` (VT).
- Every inventory row is adjudicated and implemented as adjudicated (VA);
  the audit re-pass finds nothing unrecorded (VA).
- No skill violates the restate line, save ownerless tables logged in IMP-500
  (VA).
- `doctrine check gate` green at close.

## Summary

Make the reference library the owner it is now declared to be, and make every
citation of it recognisable and checkable.

## Follow-Ups

- S2 (memories as curriculum), S3 (human set), S4 (wider enforcement) —
  RFC-033.
