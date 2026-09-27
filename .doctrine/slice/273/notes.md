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
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
