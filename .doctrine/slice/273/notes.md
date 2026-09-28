# Notes SL-273: Library ownership and lib: citations

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (exploring, 2026-09-27)

Evidence: `research/research.md` (gitignored; three threads, verified rows ✓).

**Constraining governance.** ADR-005 (owner per material type; memory line;
`lib:` invariant; restate line R-OQ-4; one workflow doc), ADR-024 (form 2
`lib:<address>`; classify before rewriting; a resolving replacement is the
evidence, absence is not), ADR-019 (published ≠ on disk), SPEC-026 / PRD-017
(manifest sole authority, stable addresses), STD-003 (unresolved address
disclosed by name), STD-001 (`lib:` marker a named constant).

**Shaping decisions (user, 2026-09-27).**
- Resolution check: one pure scanner + the publication resolver, two callers —
  a build-repo test over shipped surfaces (the sweep's exit criterion) and a
  `doctor` leg over client `.doctrine/**` text. User: "if it's that cheap, why
  not".
- Citation sweep runs on a pi worker using DeepSeek.

**Open questions (to resolve in inquiring).**
- SL-242 overlap: absorb its doctor check; amend its bare-form context line and
  the Non-Goal forbidding the sweep.
- Sweep input: positive target list (library docs cited *as* library docs),
  never basename→`lib:`; collision set (`design.md`, `notes.md`, `plan.md`,
  `memory.md`, `governance.md`, `AGENTS.md`) stays unless clearly the library.
- Rules owned only by skills (memory, backlog, knowledge gating, spec
  authoring, worktree, publication model, `doctrine.toml`): author which owners
  here?
- Stale docs: retire `boot-footer.md`; fix `shipped-corpus-authoring.md`;
  `governance.md` "loaded by boot" vs never installed.
- Consolidation extent: `routing-process.md` PUSH copies as derived.
- Restate audit depth where no owner exists.
- Phasing and executors.

**Risks.** R1 sweep false positives (collisions); R2 consolidation moves
targets under the sweep (consolidate first); R3 looser worker (check + diff
review, re-run cheap); R4 boot is an embed — edits invisible until rebuild, and
a stale-binary session regenerates old boot.

**Assumptions.** A1 library addresses stable except deliberate retirements.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · design:inquiring · 200a9f1da

### Produced
- Upstream (RFC-033 descent): frame settled 2e02a6cd2; overview memory un-inlined d947dd363; REV-067 applied to ADR-005 + ADR-024 0cf1eaf21 (lib: form); SL-144 abandoned, DEC-010 accepted
- SL-273 scoped c5a02411e; research round (gitignored `research/research.md`); triage above 9b4356984; design run at inquiring rev 10 (governance-confirmed, graph-reviewed recorded)
- minted: ISS-497 — stale-binary `memory sync` rolls back the shipped corpus

### Learned
- mem.pattern.research.pi-agents-need-tracked-background — detached pi agents die at tool-call end
- observation records: uncommitted shipped-memory edit reverted by a concurrent agent (commit immediately); QUE link-label refusals

### Open
- QUE-228 — how tier-2 project governance overrides a pulled library rule
- A1 (notes triage) — library addresses stable except deliberate retirements
- design questions inq-1..inq-8 live in the design run, not here

## Design review (reviewing, 2026-09-27)

RV-408 (codex, gpt-6-sol high): seven findings, all fix-now, all verified by
the raiser; ledger done. **Further pass:** none needed before lock. The
design's residual risk sits in execution, not design — the scanner grammar
and the per-occurrence inventory check are proven by phase 1 tests and the
phase 4 check, and the audit re-pass (DEC-346) is the planned independent look.
A further design pass would only re-probe the scanner grammar (sections 3.1,
4.4), which F-3/F-7 already exercised twice.

## PHASE-01 VA-1

`./target/debug/doctrine doctor` on this repo (built from the PHASE-01 delta), exit 0. The leg is wired at `src/commands/doctor.rs:90` (`lib_citation_findings`); no bare report is emitted. `[Lib Citation]` section, verbatim:

```text
[Lib Citation]
  warning: .doctrine/rfc/033/rfc-033.md:138: lib:reference/ — malformed address (not a safe relative logical path); `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/design.md:233: lib:reference/glossary.md#x — not declared in the publication manifest; `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/design.md:234: lib:reference/glossary.md?x — not declared in the publication manifest; `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/design.md:475: lib:templates/… — not declared in the publication manifest; `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/plan.md:59: lib:reference/glossary.md#x — not declared in the publication manifest; `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/plan.md:59: lib:reference/x.md — not declared in the publication manifest; `doctrine library tree` lists what exists
  warning: .doctrine/slice/273/research/raw/thread-2-census.md:145: lib:reference/x.md — not declared in the publication manifest; `doctrine library tree` lists what exists
```

The expected design examples appear (`#x`, `?x`, `reference/x.md`). Also present are `lib:templates/…` (a design 7.2 example) and a malformed `lib:reference/` in RFC-033. Findings reached through the slug symlink `273-library-ownership-lib-citations` and through the `phases` link into `.doctrine/state/` are deduplicated and skipped (canonical-path walk).
