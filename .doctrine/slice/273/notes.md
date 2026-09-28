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

## PHASE-02 Part B

B1 to B3 landed in commits bb223bb37 (approve REV-069), e619b6be3 (apply) and 28835a97a (pure rename). This section covers B4 to B8.

### B4: essentials compaction (VA-2)

`wc -l install/essentials.md` gives **88**, down from 90 and within I4's budget. The changes:

- A new first line states the role: the compact summary every session carries, which owns routing, core process, guardrails and the library register, while each other block cues its owner.
- Reference forms are now a per-message summary: durable ids, bare doc-local ids with first-use qualification, and criteria modes. The long C5 text is cut, since A2 moved it. The block ends with `lib:reference/glossary.md` § reference forms.
- The guardrails' storage-tier clause ends with `lib:reference/using-doctrine.md`.
- The "Reference docs" block keeps the rule and the register, and cues `lib:reference/using-doctrine.md` § publication for the model.
- The core-process paragraph was reflowed with no word changes, which saves one line. One of the doubled blank lines was dropped.

Push test outcome: no line that passes it was cut. The transition clause ("a doc still cited bare as `<name>.md` is at `reference/<name>.md`") was **kept**. It still passes the push test while skills carry bare citations, and it can go once PHASE-03/04 converts them (I2). The pinned strings are intact: `Route before you act`, `Reference forms`, `Reference docs` with `using-doctrine.md`, `**Core process:**`, `**Guardrails:**`, `doctrine library show <citation>`, and `mandatory, not optional reading`.

### B5: `using-doctrine.md`

The intro's "see the routing digest" became `lib:reference/essentials.md`. In § Pointers, the register duplicate (the bare glossary line) became `lib:reference/essentials.md`, the register's owner. The `--help` pointer is kept. I3: the only citations of essentials from an owner doc are these two, and both point at blocks essentials owns (workflow and register), not at a summary.

### B6: memory repair

- Shipped: `mem_019ec92b…:13` now names `essentials.md`, and `mem_019e9a12…:8` now names `## Essentials`. Then `cargo build` and `memory sync -y` ran (2 changed).
- Local scopes: `mem_019ea473…`, `mem_019ea4f1…`, `mem_019ed43e…` and `mem_019ed3fa…` (glob, title and body) were repointed to `install/essentials.md`.
- Local prose: `mem_019ea473…` lines 5 and 17, `mem_019ef1ae…` lines 56 and 120-121, `mem_019f2221…:19` and `mem_01a0d8f1…:5`.
- `memory edit` resets verification to `unverified` on the edited memories. Re-attestation is left to the orchestrator.
- Retrieve control: in this checkout it surfaces only the shipped signpost. The checkout has **no git remote**, so its repo identity is `repo:git-root:04b0…`, and retrieve filters out all 1178 items scoped to `github.com/davidlee/doctrine`. That includes the four targets, and even `--path-scope CLAUDE.md` returns nothing. The control was re-run in a scratch repo with origin `github.com/davidlee/doctrine` and a copy of `.doctrine/memory/items`. There, `--path-scope install/essentials.md` surfaces all four (`mem_019ea473…`, `mem_019ea4f1…`, `mem_019ed3fa…`, `mem_019ed43e…`), and `--path-scope install/routing-process.md` surfaces none of them (negative control).
- `doctrine install -y` modified no tracked files: git status was identical before and after.

### B7: old-name sweep (VA-3)

The A0 command was re-run verbatim.

| | files | hits |
|---|---|---|
| A0 baseline (HEAD 9816f05b9) | 58 | 129 |
| B7 final | 42 | 108 |

Repaired (17 files gone from the result): ADR-024, SPEC-011, three of ADR-005's hits, the 7 local memory items (9 files), both shipped `memory/` masters and their 2 synced copies, `publication/manifest.toml`, 7 of `src/boot.rs`'s hits, and `tests/e2e_claude_install.rs`. REV-069 is new since A0.

Remaining hits and their dispositions:

```text
3  .doctrine/adr/005/adr-005.md            history (Neutral line records the rename; R-OQ-3, R-OQ-5 resolved OQs, D3)
4  .doctrine/adr/005/inquisition.md        history
13 .doctrine/revision/069/revision-069.md  history (Before blocks + rationale of the applied Revision)
5  .doctrine/backlog/chore/023             history
2  .doctrine/backlog/chore/043             history
2  .doctrine/backlog/chore/078             history
1  .doctrine/backlog/idea/030              history
1  .doctrine/backlog/idea/055              history
4  .doctrine/backlog/improvement/159       history (open; see note)
6  .doctrine/backlog/improvement/160       history (closed)
1  .doctrine/backlog/improvement/306       history
2  .doctrine/backlog/improvement/315       history
2  .doctrine/backlog/improvement/345       history
3  .doctrine/backlog/improvement/376       history
2  .doctrine/backlog/improvement/435       history
1  .doctrine/backlog/improvement/458       history
1  .doctrine/backlog/improvement/484       history
2  .doctrine/backlog/improvement/488       history
2  .doctrine/backlog/issue/309             history
1  .doctrine/backlog/issue/444             history
2  .doctrine/knowledge/decision/010        history
1  .doctrine/knowledge/decision/261        history
1  .doctrine/knowledge/decision/342        history
3  .doctrine/knowledge/decision/343        history
6  .doctrine/knowledge/decision/344        history
1  .doctrine/knowledge/decision/346        history
1  .doctrine/observations/records/ee/019fad6c…  history
2  .doctrine/rfc/011 (case-notes, friction-taxonomy)  history
1  .doctrine/rfc/023/rfc-023.md            history
1  .doctrine/rfc/025/rfc-025.md            history
26 .doctrine/rfc/033 (rfc + research)      history
2  .doctrine/rfc/034 (rfc + research)      history
3  src/boot.rs                             history (:128 retired boot-footer comment; :4473, :4505 VT-1 asserts the old heading's absence)
```

Backlog items record intent as it stood when they were filed, so they count as history even when open. One note: IMP-159 ("boot-footer.md: inject user-authored footer") is open, but DEC-343 retired `boot-footer.md`, so it is probably obsolete. That is for the orchestrator to triage; it is not repaired here.

### VA-1: owner map (design 5.2) against the landed docs

| concept | owner | check |
|---|---|---|
| reference forms, criteria modes, first-use (C5) | `glossary.md` § reference forms | ok. It holds entity ids, doc-local ids, first-use and criteria modes. `using-doctrine.md` § edit-preserving rules cites `lib:reference/glossary.md` § reference forms. The design, plan, spec-tech and spec-product templates each carry one `lib:reference/glossary.md` cue. Essentials has a summary and a cue. |
| storage tiers, read via `show` | `using-doctrine.md` § reading entities, § storage tiers | ok. The essentials guardrails keep the summary and cue `lib:reference/using-doctrine.md`. |
| publication model, `lib:` form | `using-doctrine.md` § publication | ok. Essentials carries the push rule and cues § publication. `shipped-corpus-authoring.md` cites it (Part A). |
| reference-docs register | `essentials.md` "Reference docs" | ok. The `using-doctrine.md` § pointers duplicate is cut to `lib:reference/essentials.md`. |
| authority ranking | `authority.md` + `authority-model.md` | ok. Untouched, and both are published. |
| review ledger, harvest, dispatch mechanics | their docs | deferred to the section 6 restate audit (PHASE-03 onward). Not in this phase. |

I3: no owner points back to an essentials summary.
