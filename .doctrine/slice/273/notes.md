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

## PHASE-03 worker brief

Verbatim copy of the sheet sections "Inventory roots and token classes",
"`inventory.toml` row schema", "Recommendation criteria" and "Worker brief"
(`.doctrine/state/slice/273/phases/phase-03.md`), appended by W-C to satisfy EN-2
before enumerating C- rows (T1).

## Inventory roots and token classes

**Roots.**

| root | C- rows | R- rows | enumeration |
|---|---|---|---|
| `plugins/doctrine/skills/**` (every file) | yes | yes | regex + full read (R) |
| `install/**` (every file: md, toml, just, hooks, hymns, design-prompts, agents, templates) | yes | only `install/templates/plan.toml` comment | regex |
| `src/**` prose-bearing text (A3) | yes | no | literal grep + `--help` walk |

Use `rg --no-ignore --hidden`: the check walks every file under the roots, so
gitignore must not hide any.

**Token classes to enumerate (C-).**

1. **`*.md` tokens**, verbatim, as matched by
   `MD='[A-Za-z0-9_.<>{}*/~-]*[A-Za-z0-9_>}*-]\.md\b'`. This is one regex for
   bare basenames (`glossary.md`), paths (`install/glossary.md`,
   `.doctrine/governance.md`, `reference/harvest.md`), globs (`**/handover.md`)
   and placeholders (`<name>.md`, `slice-NNN.md`). The `token` field is the
   regex match exactly: `lib:reference/glossary.md` yields token
   `reference/glossary.md` (`:` is outside the class).
2. **Library path forms that don't end in `.md`**:
   `(reference|templates)/[A-Za-z0-9_.<>{}-]+` not preceded by
   `[:A-Za-z0-9_/-]`, e.g. `reference/doctrine.toml.example`,
   `templates/plan.toml`. Baseline: **0** in plugins+install. Still run it.
3. **`doctrine library show` forms**, including the **wrapped** form, where
   `library show` ends one line and the address opens the next. Five library
   maintainer headers use it (`install/{review-ledger,dispatch-mechanics,
   using-doctrine,claude-activation,harvest}.md:2-3`). The token is the
   address (class 1 catches it). The excerpt must span both lines so PHASE-04
   can rewrite the whole form. Probe:
   `rg -n 'library show`?\s*$|library\s*$' plugins/doctrine/skills install`.
4. **Src prose** (A3), in two sub-steps:
   - `--help` walk: for each command and verb, `doctrine <c> [<v>] --help`,
     grep `\.md\b|reference/|library show`, then map each hit to its doc
     comment's `file:line`.
   - Non-test literals: per file, stop at the first `#[cfg(test)]`, skip
     `tests.rs`, and grep string literals (including continuation lines,
     `concat!` and raw strings) for `MD`. Keep only those that reach output.

**Restate offenders (R-)**, one row per offending **block** (a table, a list,
or a paragraph), with non-overlapping spans:

- `flag-shape`: a flag table, an option list, an enum/value table for flags,
  or prose explaining what flags do or which values they take. Owner:
  `doctrine <cmd> --help`. Recommend `cut-to-help`.
- `owned-concept`: prose or a table restating a concept a library doc owns.
  Owners per design 5.2 and research overlaps:
  - `lib:reference/review-ledger.md`: RV acts, severity/disposition vocabulary
  - `lib:reference/harvest.md`
  - `lib:reference/dispatch-mechanics.md`: the funnel, oracle, unbound fork,
    worktree verb classes
  - `lib:reference/glossary.md`: reference forms, criteria modes
  - `lib:reference/using-doctrine.md`: storage tiers, read via `show`,
    publication, which-home
  - `lib:reference/authority.md`

  Recommend `cut-to-lib`.
- `ownerless-concept`: a concept table with no library owner. Map it to an
  IMP-500 gap:
  - G1 push/pull tiering and boot anatomy
  - G2 memory mechanics
  - G3 backlog mechanics
  - G4 knowledge gating
  - G5 spec authoring
  - G6 worktree isolation
  - G8 `doctrine.toml` schema

  Recommend `log`.
- **Not an offender** (not recorded): a single inline invocation at the step
  where the agent runs it, e.g. "flip the phase with
  `doctrine slice phase <id> PHASE-NN --status in_progress`", naming a verb and
  the flags that step needs without explaining them. This is ADR-005's MAY
  (name a verb). **This boundary needs human confirmation: see OQ-1.**

## `inventory.toml` row schema

The file holds a header comment, then `[[row]]` tables. It is generated with
`nu` (`{row: $rows} | to toml`). **Never hand-escape excerpts**: nu round-trips
backticks, quotes and newlines (probed). Validate with `nu -c 'open <file>'`.

```toml
# SL-273 sweep inventory — design 6.1. Rows are append-only; ids never renumber.
[[row]]
id        = "C-014"   # C-nnn citation / R-nnn restate; 3-digit; sorted by (file, line) at mint; append-only
file      = "plugins/doctrine/skills/audit/SKILL.md"   # repo-relative
line      = 42        # 1-based line of the excerpt's first line; informational (lines drift)
token     = "review-ledger.md"        # (+) C: the MD / path regex match, verbatim; R: ""
excerpt   = "see `review-ledger.md` for the turn protocol"   # verbatim text; occurs EXACTLY ONCE in file; may span lines
span      = "`review-ledger.md`"      # (+) exact substring of excerpt that target/resolved replaces; occurs EXACTLY ONCE in excerpt; C: contains token
class     = "library-doc"
recommend = "convert"
target    = "`lib:reference/review-ledger.md`"   # replacement for span; "" when recommend is leave | log
owner     = ""        # (+) R: "--help:<cmd>" | "lib:reference/<doc>.md" | "IMP-500:G<n>"; C: ""
within    = ""        # (+) C: id of the R- row whose span contains this occurrence, else ""
reason    = "names the published review-ledger protocol"
verdict   = ""        # orchestrator: accept | amend | reject — empty until adjudicated
resolved  = ""        # orchestrator, on amend only: replacement for span (non-empty)
```

(+) marks a field added to design 6.1's sketch. All are additive and consistent
with its semantics (Decisions D3).

**Vocabularies** (design 6.1, unchanged):

| prefix | `class` | `recommend` |
|---|---|---|
| C- | `library-doc` · `library-path` · `template` · `slice-artefact` · `client-file` · `repo-file` · `other` | `convert` · `leave` |
| R- | `flag-shape` · `owned-concept` · `ownerless-concept` | `cut-to-help` · `cut-to-lib` · `log` |

Class meanings (C-):

- `library-doc`: a bare basename (or an existing `lib:` site) of a published
  reference doc, meant as that doc.
- `library-path`: a path form addressing a library asset:
  `reference/<x>`, `doctrine library show <x>`, `install/<x>`,
  `.doctrine/<libdoc>.md`.
- `template`: a published template (`templates/<x>`, or a template basename
  used to mean the template).
- `slice-artefact`: a client entity or runtime file: `design.md`, `plan.md`,
  `notes.md`, `phase-NN.md`, `handover.md`, `research.md`, `audit.md`,
  `inquisition.md`, `slice-NNN.md`, `review-NNN.md`, `revision-NNN.md`,
  `memory.md`, and so on.
- `client-file`: a client repo file: `CLAUDE.md`, `AGENTS.md`,
  `.doctrine/governance.md`, `README.md`, the projected
  `project-orientation.md`.
- `repo-file`: a shipped-tree or doctrine-repo file that is not a library
  address: `SKILL.md`, a skill sibling such as `rigour/reference.md` or
  `worktree/NOTICE.md`, and src path constants.
- `other`: placeholders, globs, examples.

**Outcome rule** (used by PHASE-04 and by the checks here):

- The text **changes** iff `verdict = "amend"`, or `verdict = "accept"` and
  `recommend` ∈ {`convert`, `cut-to-help`, `cut-to-lib`}.
- The new text of `span` is `resolved` on amend, and `target` otherwise.
- Every other row leaves its excerpt byte-identical. That covers `leave`,
  `log` and `reject` rows.
- A C- row with `within` set is **subsumed** when its R- row changes: PHASE-04
  checks the R- row only.
- To delete text, widen `span` so the replacement is non-empty (no sentinel).
  Example: `span = "…library. A doc still cited bare…\`reference/<name>.md\`."`
  with `resolved = "…library."`.

## Recommendation criteria (the worker applies these; they are design 6.2)

C- rows:

1. `convert` only where the text means the **published library asset**, that
   is, where `doctrine library show` would return what the sentence refers
   to. The target is a code-span `lib:` address using post-rename names
   (`lib:reference/essentials.md`, never `routing-process.md` or
   `boot-footer.md`). Section names stay as prose after the citation
   (`` `lib:reference/glossary.md` § reference forms ``). No `#anchor`.
2. `template` and `slice-artefact` rows are `leave`, unless the sentence is
   plainly about the published template (then `lib:templates/<x>`).
3. `governance.md` and `AGENTS.md` are `leave`, unless plainly the library
   copy.
4. An existing `lib:` site is `leave` (A4).
5. A library maintainer header ("read it with `doctrine library show
   reference/<self>.md`") is `convert`. The span is the whole code span,
   including the wrap, and the target names the doc's own address in `lib:`
   form. The worker proposes the wording, e.g. "read it as
   `lib:reference/<self>.md`"
   (`mem_01a0d933…`; shipped-corpus-authoring :94-103).
6. `library-path` forms with a library meaning (`install/<doc>.md`,
   `.doctrine/<doc>.md`, `library show reference/…`) are `convert`, and the
   span covers the whole path form.
7. A `.toml` `paths`/`globs` value, or any data field (not a comment), is
   `leave` and goes on the attention list.
8. Every src row is `leave` (A3). If one really does cite a library doc as
   guidance, put it on the attention list.
9. A bare library-doc basename under the test roots that must stay a filename
   is `leave` plus **I2-conflict** (S1).
10. The essentials transition clause (`install/essentials.md:80-81`, "A doc
    still cited bare as `<name>.md` is at `reference/<name>.md`.") is
    `other`/`leave`, on the attention list as **delete-at-PHASE-04**. Its
    tokens `<name>.md` and `reference/<name>.md` are placeholders.

R- rows: the class decides the recommendation (table above). The
`install/templates/plan.toml` reference-forms comment (:21-28, the part about
criterion ids) is `owned-concept` → `cut-to-lib`. Its target should mirror
the four md templates' Part A cut, one line reading
`# Reference forms: \`lib:reference/glossary.md\` § reference forms.`. Keep any
ordering or immutability sentence that is plan-specific. The `glossary.md`
token inside it is its own C- row with `within` = that R- id.

## Worker brief (self-contained; pass verbatim to W-C and W-R)

> You are a capsule-worker for SL-273 PHASE-03 (sweep inventory). You
> **enumerate and recommend**. You do **not** edit any skill, install asset
> or src file. You write only `.doctrine/slice/273/inventory.toml` and, where
> this brief says so, `.doctrine/slice/273/notes.md`. Read the design
> sections 6.1, 6.2 and 7.2 with `doctrine show SL-273`, or
> `.doctrine/slice/273/design.md:364-482`.
>
> Your context carries this sheet's sections "Inventory roots and token
> classes", "`inventory.toml` row schema" and "Recommendation criteria". They
> are binding for your task.
>
> Method:
>
> 1. Enumerate with the regexes given, using
>    `rg --no-ignore --hidden -n -o`, and read each hit in context (±2 lines).
> 2. Build the rows as a nu table. Emit them with `{row: $rows} | to toml`.
> 3. Leave `verdict` and `resolved` empty.
> 4. Run checks V1–V4 (and V5/V6 for W-C) from this sheet, and paste their
>    raw output into your hand-back.
>
> Hand back:
>
> - the row count by class and recommend;
> - an **attention list** of row ids where the 6.2 rules don't clearly decide,
>   tagged `I2-conflict` / `data-field` / `delete-at-PHASE-04` / `judgement`;
> - any STOP (S1–S4) you hit;
> - any friction you met. You cannot record observations, so report them.
>
> The enumeration is exhaustive: record every occurrence of the C- token
> classes, whatever its class. Classifying is not filtering.

## PHASE-03 adjudication

O's adjudication list, applied verbatim by W-A (task T6):

```text
default: accept
C-092 reject    # "Same-named files" illustration: the bare name IS the point; not in the bare-test output (governance.md exempt)
C-094 reject    # same illustration; design.md is a template name, not a reference target
C-024 amend excerpt="tree` lists the library. A doc still cited bare as `<name>.md` is at\n`reference/<name>.md`. A retrieval" span=<same as excerpt> resolved="tree` lists the library. A retrieval"
                # \n = one real newline. Deletes the essentials transition clause (delete-at-PHASE-04). Check that the new excerpt occurs exactly once in install/essentials.md and still contains C-024's token.
C-025 accept    # verdict accept (recommend stays leave), but APPEND to its reason: " — subsumed by C-024's amend: the whole transition clause is deleted at PHASE-04; PHASE-04 must not flag this excerpt's disappearance"
R-004 amend resolved="Prefer the MCP review tools over the CLI when your harness has them\n(`lib:reference/review-ledger.md` § Passing prose); `review unlock` and\n`review paths` stay CLI-only."
                # keeps the unowned CLI-only exception the plain cut would lose
R-502 amend resolved="   disposition and a terminal close. Hold the inquisitorial line on the\n   anti-escape guardrails, `lib:reference/review-ledger.md` § 4."
                # the plain target left a bare review-ledger.md (an I2 failure) and dropped the list indent
R-511 reject    # `doctrine memory record --help` does not carry the --lifespan enum; cutting loses information
R-512 reject    # `memory record --help` does not carry the --trust/--severity enums
R-516 reject    # `memory retrieve --help` does not carry the --lifespan filter ordering
R-006 reject    # OQ-3 provisional default: human-engagement text stays in place (owner unreachable, QUE-228 open); pending the user
R-010 reject    # OQ-3, same
```

**Counts by verdict** (337 rows total): `accept` 327, `amend` 3 (C-024, R-004,
R-502), `reject` 7 (C-092, C-094, R-511, R-512, R-516, R-006, R-010).

**Plan adaptation, recorded for PHASE-04.** R- `target`/`resolved` texts are
semantic replacements, not layout-ready. 45 cut rows are single-line and
zero-indent against spans that are multi-line and in places indented, with
widths up to 239 columns. PHASE-04 lands each replacement preserving the
span's leading indent and list context and reflowing to the file's wrap width
(≤ 80 columns). Its outcome check therefore compares whitespace-normalised
text (collapse runs of whitespace), not bytes. C-025 is subsumed by C-024's
deletion. Rows with `within` set are subsumed when their R- row changes
(R-004 and R-502 change via amend; R-511/512/516/006/010 are rejected, so any
C- row within them stands on its own verdict).

## PHASE-03 inventory checks

Run from `/work/doctrine` at the post-adjudication `inventory.toml` (337
rows, `verdict` set on every row: 327 `accept`, 3 `amend`, 7 `reject`).

**V1: TOML parses.**

```
nu -c "open $INV | get row | length"
337
```

**V2: ids and enums.**

```
bad rows: 0
uniq ids: 337 vs row count: 337
```

**V3: fields.**

```
rows with != 13 keys: 337
target-nonempty-iff-recommend mismatches: 0
R- rows with empty owner: 0
rows with within not naming an existing R- id: 0
```

Note: the sheet's V3 says "every row has all 13 keys", but the schema block
(and every row) actually carries 14 keys (id, file, line, token, excerpt,
span, class, recommend, target, owner, within, reason, verdict, resolved).
All 337 rows are uniform at 14 keys — 0 missing, 0 extra — so the substantive
check (every row has the full, consistent field set) passes; the "13" in the
sheet's prose looks like a miscount against the schema block, not a data
defect. Flagging rather than silently reconciling.

**V4: locatable** (run over all 337 rows, not just the changed ones).

```
checked: 337
excerpt failures: []
span failures: []
token failures: []
```

**V8: verdict completeness (VA-1).**

```
rows with verdict=='': 0
amend rows with resolved=='': 0
```

**V9: I1 pre-check.**

```
changing rows: 131
distinct lib addresses: 9
[reference/authority-model.md, reference/claude-activation.md,
 reference/design-run-obligations.md, reference/dispatch-mechanics.md,
 reference/glossary.md, reference/harvest.md, reference/review-ledger.md,
 reference/shipped-corpus-authoring.md, reference/using-doctrine.md]
contains routing-process: false
contains boot-footer: false

./target/debug/doctrine library show reference/authority-model.md: exit=0
./target/debug/doctrine library show reference/claude-activation.md: exit=0
./target/debug/doctrine library show reference/design-run-obligations.md: exit=0
./target/debug/doctrine library show reference/dispatch-mechanics.md: exit=0
./target/debug/doctrine library show reference/glossary.md: exit=0
./target/debug/doctrine library show reference/harvest.md: exit=0
./target/debug/doctrine library show reference/review-ledger.md: exit=0
./target/debug/doctrine library show reference/shipped-corpus-authoring.md: exit=0
./target/debug/doctrine library show reference/using-doctrine.md: exit=0

control (known good): reference/glossary.md: exit=0
control (known bad): reference/routing-process.md: exit=1
```

**V10: I2 pre-check.**

```
candidate rows (basename is a manifest target, excerpt not already lib:): 82
exceptions: 0
```

The 82 matches the EX-2 bare-test control count (V6, 82 lines at `133bd5cb7`),
consistent with A2/A4: those are exactly the bare mentions this phase tracks.

**V11: IMP-500 (VA-3).**

Non-rejected `log` rows: R-510, R-513, R-521 (R-006 and R-010 are rejected
under OQ-3 and are correctly absent from IMP-500's body).

```
FOUND: plugins/doctrine/skills/record-memory/SKILL.md:23   (R-510)
FOUND: plugins/doctrine/skills/record-memory/SKILL.md:104  (R-513)
FOUND: plugins/doctrine/skills/spec-tech/SKILL.md:33        (R-521)
```

`doctrine show IMP-500` carries a new `## Ownerless restatements left in
place (SL-273 restate audit)` section (grouped by gap: G2 ×2, G5 ×1) plus a
`### Noted, not rowed` subsection for the plan/SKILL.md VT-mandate schema and
the elicit/SKILL.md "Refresh and stop" footer vocabulary, per the sheet's T6
instructions.

