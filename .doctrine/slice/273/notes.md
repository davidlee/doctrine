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

## PHASE-02 old-name sweep

A0 baseline (EX-8 positive control), captured before any Part A repair on 2026-09-28 at HEAD 9816f05b9:

```sh
grep -rnI -e 'routing-process' -e 'boot-footer' -e 'Routing & Process' .doctrine memory install plugins src tests publication | grep -v -e '^\.doctrine/slice/' -e '^\.doctrine/review/' -e '^\.doctrine/state/' -e '^\.doctrine/workflows/'
```

Result: 58 files, 129 hits. Known positives present: `publication/manifest.toml` (4, including 111 and 471), `src/boot.rs` (10, including 111), and `memory/mem_019ec92b0ffc79d294a49559db9aa12a/memory.md` (1, line 13). `install/` has 0 hits: the digest names neither itself nor the footer. `.doctrine/memory/shipped/` is gitignored, but the grep reaches it, so its two copies are counted.

Hits per file:

```text
6 .doctrine/adr/005/adr-005.md
4 .doctrine/adr/005/inquisition.md
1 .doctrine/adr/024/adr-024.md
5 .doctrine/backlog/chore/023/backlog-023.md
2 .doctrine/backlog/chore/043/backlog-043.md
2 .doctrine/backlog/chore/078/backlog-078.md
1 .doctrine/backlog/idea/030/backlog-030.md
1 .doctrine/backlog/idea/055/backlog-055.md
2 .doctrine/backlog/improvement/159/backlog-159.md
2 .doctrine/backlog/improvement/159/backlog-159.toml
4 .doctrine/backlog/improvement/160/backlog-160.md
2 .doctrine/backlog/improvement/160/backlog-160.toml
1 .doctrine/backlog/improvement/306/backlog-306.md
2 .doctrine/backlog/improvement/315/backlog-315.md
2 .doctrine/backlog/improvement/345/backlog-345.md
3 .doctrine/backlog/improvement/376/backlog-376.md
2 .doctrine/backlog/improvement/435/backlog-435.md
1 .doctrine/backlog/improvement/458/backlog-458.md
1 .doctrine/backlog/improvement/484/backlog-484.md
2 .doctrine/backlog/improvement/488/backlog-488.md
2 .doctrine/backlog/issue/309/backlog-309.md
1 .doctrine/backlog/issue/444/backlog-444.md
2 .doctrine/knowledge/decision/010/record-010.md
1 .doctrine/knowledge/decision/261/record-261.toml
1 .doctrine/knowledge/decision/342/record-342.toml
3 .doctrine/knowledge/decision/343/record-343.toml
1 .doctrine/knowledge/decision/344/record-344.md
5 .doctrine/knowledge/decision/344/record-344.toml
1 .doctrine/knowledge/decision/346/record-346.toml
2 .doctrine/memory/items/mem_019ea47314bd72c2b1f720017449a909/memory.md
1 .doctrine/memory/items/mem_019ea47314bd72c2b1f720017449a909/memory.toml
1 .doctrine/memory/items/mem_019ea4f1d1ab7d508d95eaa31651143e/memory.toml
1 .doctrine/memory/items/mem_019ed3fa2e0c7301a889a346105d3011/memory.md
2 .doctrine/memory/items/mem_019ed3fa2e0c7301a889a346105d3011/memory.toml
1 .doctrine/memory/items/mem_019ed43e279d70e3b55af747dfbfb54b/memory.toml
2 .doctrine/memory/items/mem_019ef1ae52c27ac2867b91db044f62a1/memory.md
1 .doctrine/memory/items/mem_019f2221b5327cd19c6ea1ace68a7689/memory.md
1 .doctrine/memory/items/mem_01a0d8f19c4a7e619a99a5b970e09c7e/memory.md
1 .doctrine/memory/shipped/mem_019e9a1200b6796094f9e31ff1666390/memory.md
1 .doctrine/memory/shipped/mem_019ec92b0ffc79d294a49559db9aa12a/memory.md
1 .doctrine/observations/records/ee/019fad6c-b50a-7d41-9987-bf5498c04eee.toml
1 .doctrine/rfc/011/2026-07-20.case-notes.md
1 .doctrine/rfc/011/friction-taxonomy.md
1 .doctrine/rfc/023/rfc-023.md
1 .doctrine/rfc/025/rfc-025.md
3 .doctrine/rfc/033/research/raw/thread-1-governance.md
8 .doctrine/rfc/033/research/raw/thread-2-census.md
4 .doctrine/rfc/033/research/raw/thread-3-history.md
10 .doctrine/rfc/033/research/research.md
1 .doctrine/rfc/033/rfc-033.md
1 .doctrine/rfc/034/research.md
1 .doctrine/rfc/034/rfc-034.md
1 .doctrine/spec/tech/011/spec-011.md
1 memory/mem_019e9a1200b6796094f9e31ff1666390/memory.md
1 memory/mem_019ec92b0ffc79d294a49559db9aa12a/memory.md
4 publication/manifest.toml
10 src/boot.rs
2 tests/e2e_claude_install.rs
```

## PHASE-02 Part A

REV-069 ("ADR-005 names essentials.md as the boot onboarding summary"), which originates from RFC-033, is drafted and left `proposed` / approval `none`. It has three `modify` rows: ADR-005 (primary), ADR-024 and SPEC-011. The `SPEC-011` target was accepted. Each Before block was checked verbatim against its target file. **It is awaiting VH-1**, the user's approval of the change set. No agent records that approval.

Landed in Part A:

- `install/glossary.md` § reference forms now owns the first-use qualification rule (C5). The digest copy stays until B4.
- The header comments of four templates (design, plan, spec-tech, spec-product) are cut to a `lib:reference/glossary.md` citation. `plan.toml` is left for the inventory.
- `install/using-doctrine.md` has a new `## Publication` section, and its edit-preserving rules cite `lib:reference/glossary.md` § reference forms.
- `install/shipped-corpus-authoring.md` form 2 is now a `lib:` citation that cites `lib:reference/using-doctrine.md` § publication; disposition step 3 and its table row repoint to a `lib:` address. One extra fix: its "reference-form headers in the entity templates" illustration was made stale by the template cut, so it now names the glossary's reference-form tables.
- `install/boot-footer.md` and its manifest entry are removed (EX-3).
- The digest's "Reference docs" paragraph (A7) carries the `lib:` rule, the mandatory-retrieval line and a `lib:` register. It also has a transition clause ("a doc still cited bare as `<name>.md` is at `reference/<name>.md`") for the period before PHASE-03 converts the bare citations; B4 may drop it. The file is 90 lines, and B4 compacts it to 88 or fewer.

Doctor after Part A: exit 0. `[Lib Citation]` holds PHASE-01's VA-1 set, plus nine hits at `notes.md:81-90`. Those nine come from the PHASE-01 VA-1 block above, which quotes the warnings verbatim and so re-triggers them. No file touched in Part A raises a warning.
