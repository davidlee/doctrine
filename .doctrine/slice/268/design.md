<!-- doctrine:section sec-1 -->
## What changes and why

SL-268 is RFC-032's first slice. It fixes the RV ledger's authored schema and
its derivation rules, and every later slice in the programme reads the result.
Four defects drive it:

- **Reasoning is ephemeral.** `contest`/`verify` `--note` lands in the runtime
  baton's `handoff` log and is lost with it (`ISS-280`). A stale response has no
  correction verb, and a mistaken verify cannot be reopened (`ISS-485`).
- **`done` lies.** An empty ledger derives `done` (`ISS-314`, `ISS-366`, against
  ADR-007 D-C8), and a non-empty one reads `done` mid-pass once every finding
  happens to be terminal.
- **Bad vocabulary is laundered.** An unknown `status` becomes `open`, which
  also lets `dispose` fire on a corrupted row. An unknown `severity` silently
  stops gating (`CHR-001`).
- **Prose passes through the caller's shell** (obs `019fc0bc`: live API keys
  spliced into a committed ledger; `IMP-377`).

The slice also moves the ledger's schema and derivation below the command tier,
so later slices can read an RV's status from engine code (`IMP-433`, D14's
precondition). It does this behind a black-box golden that is written first
(`IMP-029`).

Decision ids `D1`–`D15` are RFC-032 `decision-frontier.md`'s, and the frontier is
evidence (tier 5). This design adopts the frontier's user-settled choices (§5)
and the six decisions from this run's inquiry (DEC-317 to DEC-322). Once
locked, this document is the binding schema until the tech spec is authored
(DEC-321).

<!-- doctrine:section sec-10 -->
## Current state

Research baseline 2026-09-26, `research/research.md`. Its ✓ rows were
re-checked at `694acaf43`, and the symbols below were re-read for this draft.

- `src/review.rs` (5 970 lines, `review = "command"` in `layering.toml`) holds
  everything: the clap `ReviewCommand`, the vocabulary enums (`FindingStatus`,
  `Severity`, `Role`, `ReviewStatus`, `Await`, `Verb`, `TurnAct`), the
  transition table `can` and its twin `required_for`, the lenient reader
  (`FindingRow`, `ReviewDoc`, `parse_finding_status`), `derived_status`, the
  three blocker predicates, `PassFacts`, the turn seam `with_turn_hooked`, the
  baton and lock, the verbs, prime and its cache, and the renderers.
- `derived_status(&[FindingState])` reads findings only. An empty ledger gives
  `(Done, None)`.
- `ReviewMeta.concluded: bool` (`serde(default)`) is latched by `run_conclude`
  inside `with_turn`. `PassFacts.concluded` exposes it to the design run
  (DEC-138). `conclude` takes no basis.
- `parse_finding_status` maps any unknown value to `Open`. `finding_status_of`
  uses it too, so the per-finding gate treats a corrupted row as `open`.
- The three blocker predicates filter `Severity::parse(..) == Ok(Blocker)` (two
  of them) or `Err(_) => continue` (`outstanding_by_severity`). An unknown
  severity gates nothing.
- `Baton { awaiting, authored_hash, rounds, contests, handoff }` is runtime
  state. `with_turn_hooked` bumps `rounds`/`contests`. `run_raiser_transition`
  appends the `--note` to `handoff` **after** `with_turn` returns, in a second
  baton write outside the lock (`CHR-001`).
- `parse_role` accepts only `raiser`/`responder`. `disposition` is free text,
  and the design-run route axis is a `route:<route>` prose token that nothing
  validates.
- `run_prime` bails on a non-slice target and on zero selectors (`IMP-259`). A
  literal selector passes through unfiltered, so a directory or a symlink to one
  aborts hashing (`ISS-059`).
- `input::resolve_body` (engine tier) resolves `-` to stdin for
  `memory`/`knowledge edit`. No `@path` convention exists anywhere.
- The MCP `review_*` tools (`src/mcp_server/tools.rs`) call the same `run_*`
  functions with literal JSON strings.
- `layering.toml:48` says `ledger = "leaf"  # review ledger types`, but
  `src/ledger.rs` is the dispatch run-ledger (research X1).
- ADR-001's gate (`tests/architecture_layering.rs`) records edges at
  **top-level-module** granularity. It skips the out-edges of sub-classified
  umbrellas and attributes every `crate::review::…` import to `review`.

<!-- doctrine:section sec-11 -->
## Forces and constraints

Binding (confirmed at `governance-confirmed`):

- **ADR-001.** Tiers are leaf ← engine ← command with no cycles. New modules go
  in `layering.toml` and must pass `tests/architecture_layering.rs`.
- **ADR-007.** D-C5, D-C8 and D-C10 are amended by this slice's REV. D-C0–C4a,
  C6, C7, C9a/b and C11 are preserved. The baton lock and the two CAS windows
  (D-C4a) are unchanged.
- **DEC-138.** The design run's gate reads `PassFacts` (`concluded`,
  `undisposed_blockers`, `outstanding`), never `derived_status`. D2 must not
  reroute it.
- **DEC-233.** Keep RV's `Unavailable` arm here (DEC-318).
- **STD-003.** A degraded read is tolerated *and* disclosed on the channel the
  command's own caller sees (DEC-319).
- **STD-001.** Closed vocabularies are single-sourced constants with lockstep
  canaries (`*_known_set_matches_variants`).
- **SPEC-003 / REV-035.** Adding a container revises SPEC-003's inventory in the
  same change (DEC-317).
- **POL-001/002.** Plain prose. No host-project conventions in the mechanism
  (the heredoc advice is guidance, not mechanism).
- **AGENTS.md behaviour-preservation gate.** For the split, the existing suites
  plus the new golden are the proof, and they stay green unchanged.

Forces:

- **Cheap reads over pure derivation.** State stays stored and the journal is
  history (D1(b)). No read path folds turns.
- **No migration, no backfill.** Every new authored key is `serde(default)`, and
  absence has one defined meaning.
- **CLI and MCP must not diverge (R3).** Both surfaces call the same `run_*`
  functions with already-resolved strings.

<!-- doctrine:section sec-12 -->
## Module split (D4)

**A new top-level module, `src/review_ledger/`, at engine tier.** It is not a
`review::ledger` submodule. ADR-001's gate attributes edges at top-level-module
granularity, so a sub-classified `review::ledger = "engine"` would neither be
*checked* as engine (sub-classified umbrellas' out-edges are skipped) nor be
*importable* from engine code: `crate::review::ledger::…` reads as an edge into
`review = "command"`, which D14's engine-tier consumers would trip. A top-level
unit is enforced in both directions. The name avoids the dispatch `ledger`
(research X1).

`review_ledger/` (engine; imports leaf `kinds`, `tomlfmt`, `estimate`,
`value`, `listing`, and engine `relation`; nothing from command):

| file | holds |
|---|---|
| `mod.rs` | re-exports; module doc |
| `vocab.rs` | `FindingStatus`, `Severity`, `Role`, `ReviewStatus`, `Await`, `Disposition`, `Route`, `TurnKind`, their `*_KNOWN` constants and canaries |
| `schema.rs` | `ReviewDoc`, `ReviewMeta`, `Target`, `FindingRow`, `TurnRow`, the lenient readers, `read_review`/`read_reviews` |
| `derive.rs` | `derived_status`, the `rounds`/`contests` counters, `vocabulary_defects` |
| `transition.rs` | `Act`, `can` (the single transition table, `required_for` folded in), the edit-preserving writes (`apply_act`, `append_finding`, `append_review_turn`) |
| `gate.rs` | `doc_unresolved_blockers`, `undisposed_blockers`, `outstanding_by_severity`, `PassFacts`, `read_pass_facts`, `observe_pass`, `unresolved_blockers_for`, `relation_edges`, `derived_status_string` |

`src/review/` (command; `review = "command"`, one unit, not sub-classified):

| file | holds |
|---|---|
| `mod.rs` | `dispatch`, `ReviewOutput`, `ReviewError`, `print_review` |
| `cli.rs` | `ReviewCommand` (clap); resolves prose arguments through `input::resolve_prose` before calling a verb |
| `turn.rs` | `with_turn`/`with_turn_hooked`, baton, lock, `resolve_review_root` |
| `verbs.rs` | `run_new`/`mint_review`/`materialise_review_at`, `run_raise`…`run_conclude`, `run_amend`, `run_reopen`, `parse_role` |
| `read.rs` | `run_show`, `run_list`, `run_status`, the finding index (IMP-490's view struct moves here unchanged) |
| `prime.rs` | cache, `run_prime`, selector resolution |

The file boundaries are this design's commitment. Private helper placement
within them is an implementation detail.

**Sequencing.** Phase 1 writes the golden. Phase 2 performs the move with no
behaviour change. External callers (`catalog/scan.rs`, `slice.rs`,
`commands/{design,guard,show}.rs`, `priority/partition.rs`, `mcp_server/tools.rs`,
`main.rs`, `relation.rs` tests) move to `crate::review_ledger::…` for engine items
and stay on `crate::review::…` for verbs. No re-export shim is left behind:
one path per item.

**Layering map.** Add `review_ledger = "engine"`. Keep `review = "command"`.
Correct `ledger = "leaf"`'s comment to "dispatch run-ledger (SL-064)".
`catalog::scan`'s entry keeps reaching `review` (it calls
`derived_status_string` and `relation_edges`, which now live in
`review_ledger`), so its comment is updated. Whether `catalog::scan` still needs
`review` at all is checked at phase 2, and the entry is updated to match.

**DEC-233 (DEC-318).** `DERIVED_STATUS = [RV]` stays. Its comment and the
pinning test's wording change to: *derivation is engine-tier
(`review_ledger::derive`); the probe does not call it yet; the arm retires with
D14 (slice 2).*

<!-- doctrine:section sec-2 -->
## Ledger schema v2 (D1, D2 write side, D8 route)

```toml
[review]
facet = "code-review"
raiser = "codex"                # labels double as --as aliases (D8)
responder = "claude"
concluded = true                # examination closed; raise/reopen clear it; PassFacts reads it (DEC-138)
rounds_base = 4                 # seeded once, at the first journalled write
contests_base = 1

[[review.turn]]                 # review-level journal: conclude only
act = "conclude"
role = "raiser"
note = "examined src/review_ledger/**, the golden, and ADR-007 D-C5"   # the --basis

[[finding]]
id = "F-3"
status = "contested"            # current state: what gates and renders read
severity = "major"
title = "…"
detail = "…"
disposition = "fix-now"         # current, closed on write
route = "demonstrate"           # current, optional, closed on write
response = "…"                  # current account

[[finding.turn]]                # append-only; order = file order within this finding
act = "raise"
role = "raiser"

[[finding.turn]]
act = "dispose"
role = "responder"
disposition = "fix-now"
route = "demonstrate"
response = "…"                  # snapshot of what this turn answered

[[finding.turn]]
act = "contest"
role = "raiser"
note = "the repair is partial: …"
```

**Turn row.** `TurnRow { act, role, note?, disposition?, route?, response? }`,
all strings on read (open vocabulary, D15). `disposition`/`route`/`response`
appear only on `dispose` and `amend` turns. They snapshot what that turn
answered, so a later re-dispose cannot erase what a contest argued against
(D1's deliberate redundancy).

**Acts and notes.**

| act | role | from → to | note |
|---|---|---|---|
| raise | raiser | ∅ → open | none (the finding's `detail` is the account); clears `concluded` |
| dispose | responder | open \| contested → answered | optional |
| amend | responder | answered → answered | **required**; new `response`; `disposition`/`route` optional (kept if omitted) |
| verify | raiser | answered → verified | optional |
| contest | raiser | answered → contested | **required** |
| reopen | raiser | verified → contested | **required**; clears `concluded` |
| withdraw | raiser | open \| answered → withdrawn | optional |
| conclude | raiser | review-level | **required** (`--basis`) |

`contest` gains a required note. It was optional and ephemeral before. That is a
deliberate CLI and MCP change, and the golden records it. `withdraw` gains an
optional `--note`, so every act has one place for its reasoning.

**One write.** Every act's turn row is appended by the same `toml_edit` edit
that moves `status`, inside `with_turn`'s CAS-guarded write. `apply_act` is the
only writer of a finding's state fields and its turn, and `append_finding` writes
the raise turn. The ephemeral note concept, `Baton.handoff`, and the post-turn
baton write in `run_raiser_transition` are deleted (`CHR-001`).

**Counters.** At the first journalled write to a ledger with no turn rows
anywhere and no `rounds_base`, `with_turn` writes `rounds_base`/`contests_base`
from the baton's current values (0 when the baton is absent). From then on:

- `rounds = rounds_base + count(all turns, finding and review level)`
- `contests = contests_base + count(turns with act = "contest")`

A ledger that has never had a journalled write (no base, no turns) reports the
baton's `rounds`. That is the only legacy dependency, and it is cosmetic. The
baton stops incrementing both counters. It keeps them as serde-default fields so
legacy values stay readable until the seed copies them.

**No migration.** Absent `turn` means an empty journal. Absent `*_base` means
not yet seeded. Absent `route` means unrouted. Pre-journal history is not
reconstructed.

**Closed write vocabularies (STD-001):**

- `Disposition` = `aligned | fix-now | design-wrong | follow-up | tolerated`.
- `Route` = `review | demonstrate | probe | control | owner-fix`.

Each has a `*_KNOWN` constant and a canary, and a refusal names the set.
`dispose`/`amend` refuse a `route:` prefix in `--disposition`, pointing at
`--route`.

<!-- doctrine:section sec-3 -->
## Derivation and fail-safe reads (D2, D15)

**Status (D2).** `derived_status(findings: &[FindingState], concluded: bool) ->
(ReviewStatus, Await)`:

| condition | status | await |
|---|---|---|
| any finding open, contested or unknown-status | active | responder |
| else any answered | active | raiser |
| else (all terminal, including empty) ∧ ¬concluded | active | raiser |
| else (all terminal) ∧ concluded | done | none |

It is derived on every read and never latched.

**What `concluded` means (RV-396 `F-4`).** `concluded` says the examination is
currently closed. `conclude` sets it, with a basis saying what was examined.
Disposition and verification may follow a conclude: they resolve findings, and
they do not reopen the examination. **`raise` and `reopen` on a concluded
ledger clear it** (`concluded = false`), in the same write that appends their
turn. So `done` needs a fresh conclude after the last raise or reopen, and
conclude is enforceably the closing move of every pass. This is checked at
write time, so no cross-finding turn order is needed. Earlier conclude turns stay
in `[[review.turn]]` as the record of each pass. This replaces frontier D2's
"a new finding does not clear the marker", on the user's 2026-09-26 ruling. A
design run's `Conducted` disposition (DEC-138, which reads
`PassFacts.concluded`) therefore stops being admissible after a late raise
until the raiser concludes again.

Every caller moves with the signature: `ReviewDoc::derived`,
`reconcile_baton_fields`, `run_status`, show/list, `derived_status_string`.
`doc_unresolved_blockers` keeps its `Active` guard. More ledgers now read
`active`, but it still counts only non-terminal blockers, so the close gate
(D-C9b) changes only where D15 widens what gates (below). The `PassFacts`
predicates and the design-run gate are unchanged (DEC-138). `PassFacts` gains
only the additive `defects` field (below).

**Unknown status (D15).** `FindingStatus::parse(&str) -> Result<FindingStatus,
Unknown>` replaces `parse_finding_status`. The derived reads hold a
`FindingState { status: Option<FindingStatus>, raw }`, where `None` is out of
vocabulary:

- it reads **non-terminal**: it keeps the review `active` and counts toward
  every blocker predicate as non-terminal;
- the transition table admits **no act** on it. `gate` refuses with
  `ReviewError::UnknownStatus { finding, raw }`, which names the known set and
  the repair (correct the value in the ledger TOML; the next turn's entry CAS
  heals the baton);
- it renders verbatim.

**Raw-preserving projection (RV-396 `F-5`).** The typed `Finding` projection
used by `show`, the finding index, `--json` and MCP carries `status` and
`severity` as `Vocab<T> = Known(T) | Unknown(raw)`. It serialises as the raw
string and renders verbatim. `finding_of_row`'s `Major` fallback and the `Open`
fallback are deleted, so no read surface ever shows a legitimate value in place
of a corrupted one.

**Unknown severity (D15).** One predicate, `gates_as_blocker(raw) -> bool` =
`Blocker` or out of vocabulary, is used by all three blocker predicates.
`outstanding_by_severity` counts an unknown severity in the `blocker` bucket.
That closes the silent non-gating, and the three predicates keep their deliberate
state differences (SL-244).

**Unknown disposition, route, act or role in a turn:** rendered verbatim. They
gate nothing and are counted as turns.

**Disclosure (DEC-319).** A pure `vocabulary_defects(&ReviewDoc) ->
Vec<VocabDefect>` names each out-of-vocabulary `status`/`severity` as
`{finding, field, raw, effect}`, for example *"F-2 severity `crit` is out of
vocabulary; gating as blocker"*. Each read surface renders it on its caller's
channel:

- `review show`/`status`: a `warning:` line per defect in `formatted`, plus a
  `warnings` array in JSON and MCP output;
- `review list`: the list output carries a structured `warnings` array, each
  entry naming the RV (RV-396 `F-2`). The CLI prints it to stderr so the table
  cells stay single-valued. `--json` and the MCP `review_list` response carry it
  as a field;
- the close gate: `BlockerRef` gains `reason: Option<String>`, and the refusal
  prints it beside the finding;
- the cross-kind catalog (RV-396 `F-3`): `derived_status_string` returns the
  status and its defects. `catalog::scan`'s `status_and_title_for` overlay
  pushes one warning `CatalogDiagnostic` per defective RV onto the scan's
  diagnostics channel, which `doctor` reads (`doctor_checks.rs:85`). Most other
  scan callers drop warning diagnostics for every kind today
  (`commands/relation.rs:331`, `commands/design.rs:1782`). That pre-existing
  STD-003 gap is ISS-492, and SL-268 does not widen into it;
- the design run (RV-396 `F-9`): `PassFacts` gains `defects:
  Vec<VocabDefect>`, and no predicate reads it. The `commands/design.rs` shell
  prints a `warning:` line per defect, naming the RV, finding, raw value and
  effect, wherever it reads `PassFacts`: the outstanding projection and gate
  admission. The `design_run` leaf types and SPEC-029 are untouched.

Out-of-vocabulary disposition and route values are not defects. D8 makes them
open on read, and the 61 legacy disposition values stay quiet.

<!-- doctrine:section sec-4 -->
## Write surface (D8, D10, new verbs, MCP parity)

**Prose arguments (D10).** `input::resolve_prose(raw, flag, stdin, fs_read) ->
Result<String>` is one engine-tier function, and `resolve_body` becomes its
`-`-only special case (its callers' behaviour is unchanged).

- `-` reads stdin in full;
- `@path` reads that file, relative to the invocation's working directory;
- anything else is literal.

A literal value that begins with `@` or is exactly `-` must come through stdin
or a file. `@` in a *target* ref (`SL-NNN@PHASE-NN`) is a different argument
and is unaffected, and the help text says so. At most one prose argument per
invocation may be `-`. A second `-` is refused before any read, naming both
flags. An empty resolved value is refused wherever the argument is required.

The review CLI shell (`review/cli.rs`) resolves `--title`, `--detail`,
`--response`, `--note` and `--basis` before calling a verb. The `run_*`
functions take resolved `String`s.

**MCP.** The `review_*` tools pass JSON strings straight to the same `run_*`
functions with no resolution. There is no shell to defend against, and a
literal `-` or `@x` stays literal. New fields:

- `review_conclude.basis` (required);
- `review_contest.note` (required);
- `review_withdraw.note`;
- `review_dispose.route`;
- `review_amend` and `review_reopen`, as new tools.

`role` accepts the aliases. R3 is closed by construction: one write function per
act, two thin adapters.

**Roles (D8).** `parse_role(token, default, meta: &ReviewMeta)` accepts:

- `raiser`/`responder`;
- the ledger's declared `raiser`/`responder` labels, as aliases.

It resolves after `read_authored`. The labels are fixed at `new`, so reading
them outside the lock is race-free. `review new` refuses:

- labels that collide with each other;
- a label that names the *other* role.

`--as` help names the legal values ("`raiser` | `responder`, or this ledger's
declared labels"). The refusal lists the ledger's actual labels.

**Disposition and route (D8).** `--disposition` takes a clap `value_parser`
over `Disposition`, and `--route` is optional over `Route`. The MCP input
schemas carry the same closed `enum`s. Legacy values stay readable (sec-3).

**New verbs.**

- `review amend RV --finding F-n --response … --note … [--disposition …]
  [--route …] [--as responder]`
- `review reopen RV --finding F-n --note … [--as raiser]`

Both ride `with_turn` and `apply_act`. `Verb`/`TurnAct` become one `Act` enum
(`raise | dispose | amend | verify | contest | reopen | withdraw | conclude`).
`can(act, from, role)` is the single table: `required_for` is removed, and the
state refusal reports the admissible from-set computed from `can`, so the table
and the message cannot drift. `conclude` stays outside the per-finding arm, as
today.

**Conclude (D2).** `review conclude RV --basis … [--as raiser]`. `--basis` is
required and non-empty, and it becomes the note of a `[[review.turn]]`
`act = "conclude"` row. `concluded = true` is written in the same edit. Open or
answered findings are allowed (sec-3 defines what conclude closes). `raise`
and `reopen` clear `concluded` in their own write. A later conclude appends
another turn and sets it again. `already` reports whether the flag was set when
this conclude ran.

**Target spelling (D8).** `review new --target SL-NNN@PHASE-NN` is accepted as
`--target SL-NNN --phase PHASE-NN`. Giving both `@` and `--phase` is refused.
`review list --target` accepts the same spelling, through one parser
(`review_ledger::schema::Target::parse`).

<!-- doctrine:section sec-5 -->
## Prime (D11)

`run_prime` **degrades** instead of failing when the target has no path-set:

- the target is not a slice ref (`IMP-259`), or
- the slice declares zero selectors.

In both cases, under the prime lock, it removes any `cache.toml` left by an
earlier successful prime, so `status` reports no cache instead of a stale
`current` (RV-396 `F-6`). It then returns `Primed { tracked_count: 0, degraded:
Some(reason), cleared: bool }`, and prints `primed nothing: <reason>` (plus
`removed the previous cache` when one existed) on stdout, exit 0 (STD-003:
disclosed, not silent). The MCP output carries `degraded`. A primed
slice with selectors behaves as today.

**Literal selectors (`ISS-059`).** The literal arm gets the glob arm's non-file
filter:

| literal selector | treatment |
|---|---|
| absent on disk | kept (absence ⇒ stale, R1) |
| a regular file | kept |
| a directory, or a symlink (to anything) | excluded, and listed on a `skipped non-file selector:` line in the prime output |

The check uses `symlink_metadata` on the invoking tree, so a symlink is never
followed.

D-C10 is amended to this selector model in the REV (sec-6). The prose tier and
`domain_map` are retired.

<!-- doctrine:section sec-6 -->
## Governance and guidance

**Tech spec (DEC-321).** The "Review ledger" tech spec is authored through
`/spec-tech` in a governance phase **after** the code phases:

- `c4_level = "container"`, `parent = SPEC-003`;
- no `descends_from` unless `/spec-tech` finds one (IMP-481);
- live `[[source]]` anchors on `src/review_ledger/` and `src/review/`;
- it describes what landed: schema v2, the act table, the derivation rules, the
  fail-safe reads and the prime model.

**REV.** Minted with `doctrine revision new --originates-from RFC-032`:

- ADR-007 **D-C5**: the turn journal, `amend`/`reopen`, disposition closed on
  write, and `route`;
- ADR-007 **D-C8**: `done ⇔ all findings terminal ∧ concluded`, and the conclude
  marker's role with its basis, and `raise`/`reopen` clearing it;
- ADR-007 **D-C10**: the selector-model cache, and prime degrading;
- **SPEC-003** container inventory: add the new spec, and add SPEC-029, which is
  already missing (DEC-317).

It is applied at reconcile.

**D12.** `IMP-479` (the reverse index) is closed as not needed until measured.

**Guidance, and the bounded review (DEC-320).** Phase work edits:

- `install/review-ledger.md` (`CHR-079`): acts, `amend`/`reopen`, required notes,
  `conclude --basis` as the closing move after the last raise or reopen, `--route`, aliases, `-`/`@path`, and MCP
  as the preferred write path, with a quoted heredoc on the CLI;
- `install/design-prompts/reviewing.md`: `route` becomes the `--route` field, not
  a prose token;
- `plugins/doctrine/skills/{audit,code-review,inquisition,reconcile,close}/SKILL.md`
  and `plan`/`walkthrough` where they cite the route token or `--note`.

Then a **bounded review** (VA) runs over the installed guidance: `install/**/*.md`,
`plugins/doctrine/skills/**/SKILL.md`, and the MCP `review_*` tool descriptions.
It checks every `doctrine review …` verb, flag, value and argument shape, and
every `review_*` MCP tool and field, against the built binary's `--help` and the
MCP schema. It then spot-checks other `doctrine` verbs in the files it opens.
Everything found is fixed in-slice or filed. The review leaves a committed
record listing what it covered, what it found, and how each finding was
dispositioned, as an RV ledger against SL-268. The automated check is IMP-492.

**Memories, at close.** Update `mem_019f97fcab2e77a28902371f80743605`
(verified is terminal) for `reopen`, and `mem_019fdfe379b67e53857735b88c394b52`
(done is not concluded), which D2 makes false.

<!-- doctrine:section sec-7 -->
## Surface impact

Design-target selectors (recorded via `slice selector add --intent
design-target`):

| path | change |
|---|---|
| `src/review.rs` | removed; becomes `src/review/` |
| `src/review/` | new: the command modules (sec-12) |
| `src/review_ledger/` | new: the engine module (sec-12) |
| `src/input.rs` | `resolve_prose`; `resolve_body` delegates to it |
| `src/mcp_server/tools.rs` | `review_*` schemas and arms: `basis`, `note`, `route`, aliases, `review_amend`, `review_reopen`, `warnings` |
| `src/commands/cli.rs` | `Command::Review` import path |
| `src/commands/design.rs` | import paths; prints `PassFacts.defects` warnings in the projection and gate admission |
| `src/commands/guard.rs`, `src/commands/show.rs` | import paths; `guard`'s write-class table gains `amend`/`reopen` |
| `src/main.rs` | write-class test table |
| `src/slice.rs` | close gate: import path; renders `BlockerRef.reason` |
| `src/catalog/scan.rs` | import path; the status overlay pushes defect diagnostics |
| `src/priority/partition.rs`, `src/relation.rs` | test import paths |
| `src/kinds/mod.rs` | `DERIVED_STATUS` comment (DEC-318) |
| `.doctrine/adr/001/layering.toml` | `review_ledger = "engine"`; `ledger` comment; `catalog::scan` comment |
| `tests/e2e_review_golden.rs` | new: the IMP-029 golden |
| `tests/architecture_layering.rs` | unchanged unless its fixtures name `review` |
| `install/templates/review.toml` | unchanged; new keys are written at first use, not at mint |
| `install/review-ledger.md`, `install/design-prompts/reviewing.md` | guidance (sec-6) |
| `plugins/doctrine/skills/` | pass-running skills (sec-6), plus bounded-review fixes |

The bounded review may touch further guidance files. Those are recorded as
selectors when the review names them.

Out of this slice: `src/doctor_checks.rs` (DEC-322), `src/authored_status.rs`
(D14), and SPEC-029 (slice 3).

<!-- doctrine:section sec-8 -->
## Verification

**Behaviour-preservation gate (D4).** `tests/e2e_review_golden.rs` is written
first, against today's binary, over hand-seeded fixtures with fixed dates (the
SL-030 `e2e_adr_cli_golden.rs` pattern). Output is normalised before
comparison by replacing the fixture root with a fixed `<ROOT>` token. That
covers `new`'s created directory and every path-bearing error (RV-396 `F-7`).
Nothing else is carved out. It pins, byte-exact:

- stdout and error text for every verb, including `new`, `list`, `show`,
  `show --json`, `status`, `prime`, and each role and state refusal;
- the authored ledger TOML after each write.

The split phase holds the golden and every existing suite green **unchanged**.
Each later phase that changes behaviour edits the golden in the same commit, and
the diff is the evidence of the change.

**Expected changes to pre-existing tests (D2, D1; RV-396 `F-1`).** There are
two kinds, and each one is named in its phase's notes:

- **Assertion flips.** The assertion changes because the rule changed:
  `derived_status_empty_is_done_none`,
  `derived_status_all_terminal_is_done_none`, `derived_status_total_over_enum`
  (gains the `concluded` axis), `show_renders_empty_ledger_done_and_the_edge`,
  `list_renders_empty_ledger_done_and_the_edge`, and
  `note_is_handoff_chatter_in_the_baton_not_the_ledger` (replaced by a
  note-in-turn test, since D1 retires the concept it pins).
- **Fixture-setup changes.** A test whose point is unrelated to D2 but which
  seeds an unconcluded all-terminal ledger and asserts `done` gains a
  `concluded = true` fixture line or a `conclude` step, and its assertion is
  unchanged. This covers the ledger-reading tests near `review.rs:3777`, `3805`
  and `4748`, `golden_run_status`, and any other such test found in the phase.
  In-file `golden_*` strings that render the new `done`/`active` value are
  flips, and are named as such.

Any other change to an existing assertion is a finding.

**VT (new tests):**

1. every act appends exactly one turn, in the same write as `status`, with the
   right `act`/`role`/`note` fields;
2. `amend` and `reopen` transitions, and their refusals from every other state;
3. required notes (`contest`, `amend`, `reopen`) and required `--basis` are
   refused when missing or empty;
4. counters: seeding from the baton at the first journalled write, 0 with no
   baton, `base + count` afterwards, and the legacy ledger reading the baton;
5. `derived_status` over (finding states × concluded), including empty × both;
   a conclude with open findings reaching `done` once they turn terminal;
   `raise` and `reopen` on a concluded ledger clearing `concluded`, so `done`
   needs a fresh conclude; and `PassFacts.concluded` reading `false` after a
   late raise;
6. unknown status: non-terminal, refused by every act with the named repair, and
   rendered verbatim with its warning;
7. unknown severity: gates the close gate, counts as blocker in
   `outstanding_by_severity` and `undisposed_blockers`, and is warned on
   `show`/`status`/`list`/close;
8. a legacy unknown disposition renders verbatim with no warning; closed
   disposition and route refusals name the set; a `route:` prefix is refused;
9. `-` and `@path` on every review prose argument; two `-` refused; a leading-`@`
   literal via stdin; `resolve_body` behaviour unchanged;
10. `--as` aliases, collision refusal at `new`, and help naming the values;
11. `@PHASE-NN` target spelling, and its conflict with `--phase`;
12. prime: non-slice and zero-selector targets degrade and disclose; the literal
    non-file filter;
13. MCP parity: each new field and tool round-trips through the same `run_*`;
    a literal `-` stays literal; `review_list` carries `warnings`;
14. `PassFacts` unchanged over v2 ledgers (DEC-138), and the design-run review
    suites green;
15. layering: `review_ledger` classified `engine`, and
    `tests/architecture_layering.rs` green;
16. raw-preserving projection: an unknown severity or status renders verbatim
    in the table, index, `--json` and MCP, never as `major`/`open`;
17. prime: a successful prime, then zero selectors, then prime, leaves `status`
    with no cache;
18. the catalog scan emits a warning diagnostic for a defective RV, and
    `doctor` reports it;
19. the design-run projection and gate admission print the defect warning, and
    the gate outcome is unchanged.

**VA:**

- the bounded guidance review (DEC-320) leaves its RV with every finding
  terminal and concluded;
- the tech spec's anchors resolve (`spec show --json`), per
  `mem_019fc1308b8e7b73acf9644547351373`.

**Closure.**

- `ISS-314`, `ISS-366`, `IMP-479`, `IMP-029` and the fulfilled items are closed.
- The REV is applied at reconcile.
- `doctrine check gate` is green.

<!-- doctrine:section sec-9 -->
## Risks and residuals

- **R1: split drift.** Mitigated by the golden-first gate (sec-8). The split is
  one phase with no behaviour change, and it lands before any schema edit.
- **R2: legacy ledgers flip to `active`.** Accepted (frontier §5). The close gate
  counts only non-terminal blockers. The one new close-gate effect is D15's
  unknown severity, which is the intended teeth.
- **R3: CLI/MCP divergence.** Closed by construction (sec-4): resolution happens
  only in the CLI shell, and both adapters call one `run_*` per act.
- **R4: D-C10 depends on D11.** D11 rides here (sec-5), so the REV keeps D-C10.
- **R5: `contest --note` becomes required.** This breaks existing scripts and
  skills that contest without a note. The guidance phase and the bounded review
  cover the shipped callers, and the refusal names the flag.
- **R6: journal growth.** About one short table per act. Accepted (D1). `show`
  does not render turns by default. Slice 2 owns the read projection (D5).
- **R7: alias ambiguity.** Refused at `new`. A legacy ledger whose labels
  collide accepts only the canonical role names, and the refusal says why.
- **R8: late raises clear `concluded`.** A design run whose pass was
  conducted loses admissibility on a late raise until the raiser concludes
  again. That is intended (sec-3). The guidance and the design-run reviewing
  prompt say so.
- **Residual: catalog callers drop warning diagnostics** for every kind
  (ISS-492). The RV defect reaches `doctor` and review's own surfaces.
- **Residual: the doctor checks** are IMP-492 and IMP-493, both needing IMP-491.
- **Residual: `authored_status` still returns `Unavailable` for RV** (DEC-318,
  D14).
- **Residual: hand-written turn rows** that look valid are undetectable
  (frontier D1's stated ceiling).

