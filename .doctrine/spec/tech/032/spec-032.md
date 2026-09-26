# SPEC-032: Review ledger

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See glossary.md § reference forms. -->

## Overview

The review ledger is the mechanism behind the `review` kind (`RV-NNN`): a
turn-based adversarial loop between a **raiser**, who finds, and a
**responder**, who answers, over one authored ledger per review, in one working
tree. ADR-007 holds the decisions: review as a first-class kind, the runtime
baton, the lock and CAS, derived status, the two lifecycle gates, and the
single-ref subject. This spec holds how they are built.

The container splits along the ADR-001 tier line into two top-level modules:

- **`review_ledger`** (engine tier) is the ledger itself. It holds the closed
  vocabularies (`vocab`), the authored schema and its readers (`schema`), the
  derived status, counters and vocabulary defects (`derive`), the act table and
  its edit-preserving writes (`transition`), and the predicates other kinds gate
  on (`gate`). It reaches only leaf modules (`kinds`, `listing`, `estimate`,
  `value`) and engine modules (`entity`, `relation`), so engine-tier consumers
  can import it. The name keeps it apart from `ledger`, the dispatch run-ledger.
- **`review`** (command tier) is the `doctrine review` surface over it: clap
  parsing and prose resolution (`cli`), the turn guard with its baton and lock
  (`turn`), the mint and write verbs (`verbs`), `show`, `list` and `status`
  (`read`), the reviewer-context cache (`prime`), and the structured
  `ReviewOutput` and `ReviewError` types (`mod`).

Both are registered in the ADR-001 layering map (`review_ledger = "engine"`,
`review = "command"`), which enforces the split in both directions. Other kinds
reach the ledger one way only: the slice close gate calls
`review_ledger::unresolved_blockers_for`, the design command reads
`review_ledger::read_pass_facts` / `observe_pass`, the catalog scan reads
`review_ledger::derived_status_string` and `relation_edges`, and the MCP server
(`mcp_server::tools`, the `review_*` tools) calls the same `review::run_*`
functions the CLI does.

## Responsibilities

Mirrors the structured `responsibilities` list. The container owns the authored
ledger schema and its closed vocabularies; the act table and the single write
that applies an act; the turn protocol with its lock, CAS windows and counters;
the derived status and the conclude marker; the fail-safe reading of corrupted
vocabulary and its disclosure; the gate predicates consumed by slice close and
the design run; and the command and MCP surface, including `prime`. The
sections below give the mechanism for each.

## Ledger schema

A review is two authored files, `review-NNN.toml` (the ledger) and
`review-NNN.md` (the `## Brief` prose companion, ADR-007 D-C6), plus a
`NNN-slug` alias. The ledger is read as `ReviewDoc`:

```toml
id = 12
slug = "…"
title = "…"
tags = []

[review]                        # ReviewMeta
facet = "code-review"
raiser = "codex"                # role labels; also accepted as --as aliases
responder = "claude"
concluded = true                # the raiser has concluded the current pass
rounds_base = 4                 # counter seed, written once
contests_base = 1

[[review.turn]]                 # review-level journal: conclude only
act = "conclude"
role = "raiser"
note = "…"                      # the --basis

[target]                        # Target: RV-NNN ──reviews──▶ ref
ref = "SL-024"
phase = "PHASE-03"              # optional scope

[[finding]]                     # FindingRow: current state
id = "F-3"
status = "contested"
severity = "major"
title = "…"
detail = "…"
disposition = "fix-now"
route = "demonstrate"
response = "…"

[[finding.turn]]                # TurnRow: append-only, in file order
act = "raise"
role = "raiser"

[[finding.turn]]
act = "dispose"
role = "responder"
disposition = "fix-now"
route = "demonstrate"
response = "…"

[[finding.turn]]
act = "contest"
role = "raiser"
note = "…"
```

- **No stored status.** A review's status and await are derived (below) and are
  never written. The one stored pass-level fact is `concluded`: a pass
  finishing is an event and cannot be derived, because a clean pass and a pass
  never run present the same findings.
- **Current state and journal.** A `[[finding]]` row carries the state that
  gates and renders read. Its `[[finding.turn]]` rows are the append-only record
  of the acts that produced it. A turn row is `act` and `role`, then `note`,
  `disposition`, `route` and `response`, each present only when the act set it.
  `dispose` and `amend` turns snapshot the effective disposition, route and
  response, so a later answer cannot erase what an earlier contest argued
  against. `[[review.turn]]` journals the acts that move no finding, which is
  `conclude` alone.
- **Identity is append-only.** Findings are never deleted or renumbered. A new
  finding takes `F-<max+1>` over the existing `F-n` ids (`next_finding_id`), and
  is located by its `id`, never by array position.
- **Absence has one meaning.** Every key added after the first ledger shape is
  `serde(default)`: an absent `concluded` is not concluded, an absent `turn` is
  an empty journal, an absent `*_base` is not yet seeded, and an absent `route`
  is unrouted. No ledger is migrated, and pre-journal history is not
  reconstructed.
- **Raw on read.** `FindingRow` and `TurnRow` hold every closed-vocabulary field
  as the authored string. Each consumer classifies it through `Vocab<T>` (see
  Fail-safe reads), so a hand-edited value is carried, never rewritten.
- **Target.** `[target]` is a single canonical ref plus an optional phase
  (ADR-007 D-C11). `Target::parse` accepts `REF@PHASE-NN` as the same pair as
  `--target REF --phase PHASE-NN`, and refuses both spellings together or a
  malformed `@`.

`read_review` and `read_reviews` read ledgers as data. `read_authored` returns
the bytes as well as the parsed document, as the snapshot the CAS windows compare
against.

## Closed vocabularies

| vocabulary | values | written by |
|---|---|---|
| `Facet` | `scope` `design` `plan` `phase-plan` `implementation` `code-review` `reconciliation` | `new` |
| `FindingStatus` | `open` `answered` `contested` `verified` `withdrawn` (the last two terminal) | the act table |
| `Severity` | `blocker` `major` `minor` `nit` | `raise` |
| `Disposition` | `aligned` `fix-now` `design-wrong` `follow-up` `tolerated` | `dispose`, `amend` |
| `Route` | `review` `demonstrate` `probe` `control` `owner-fix` | `dispose`, `amend` (optional) |
| `Role` | `raiser` `responder` | every act's turn row |
| `ReviewStatus` | `active` `done` | derived, never written |
| `Await` | `raiser` `responder` `none` | derived, never written |

Each vocabulary is one enum with an `as_str` render mirror and a known-set
constant (`FACETS`, `FINDING_STATUSES`, `SEVERITIES`, `DISPOSITIONS`, `ROUTES`,
`ROLES`, `REVIEW_STATUSES`), kept in lockstep by a canary test (STD-001). Every
refusal names its set from the constant. `Disposition::parse` refuses a
`route:` prefix outright and points the caller at `--route`. The CLI's
`value_parser`s and the MCP input schemas' `enum`s are built from the same
parsers and constants, so both write paths are closed the same way.

Disposition and route are **closed on write, open on read**: a legacy
free-text value in an existing ledger stays readable and is rendered verbatim.
It is not a defect and gates nothing. Status and severity are also read without
a fallback, but they gate, so they are read fail-safe (below).

## Act table

`transition::Act` is the one act vocabulary. It serves the role gate, the
transition table, the refusal and the journal's `act` string.
`transition::can(act, from, role)` is the single transition table: pure, total,
and `const`.

| act | role | from → to | note | other fields |
|---|---|---|---|---|
| `raise` | raiser | (none) → `open` | none; `detail` is the account | `severity`, `title`, `detail` (fixed thereafter); clears `concluded` |
| `dispose` | responder | `open` \| `contested` → `answered` | none; `response` is the account | `disposition` required; `route` optional, an omitted route keeps the current one |
| `amend` | responder | `answered` → `answered` | **required** | new `response` required; `disposition` and `route` optional, kept if omitted |
| `verify` | raiser | `answered` → `verified` | optional | — |
| `contest` | raiser | `answered` → `contested` | **required** | — |
| `reopen` | raiser | `verified` → `contested` | **required** | clears `concluded` |
| `withdraw` | raiser | `open` \| `answered` → `withdrawn` | optional | — |
| `conclude` | raiser | review-level; no finding moves | **required** (`--basis`) | sets `concluded = true` |

- **One table, one refusal.** `Act::required_role` is the static half of `can`.
  `admissible_from(act)` computes the from-set a state refusal reports
  (`ReviewError::StateMismatch`) by filtering the status vocabulary through
  `can`, so the table and its message cannot disagree. `withdrawn` has no exit,
  and `verified` exits only through `reopen`.
- **Required notes** refuse with `ReviewError::NoteRequired` before the lock is
  taken, so nothing is read or written. `Act::note_flag` names the flag
  (`--basis` for `conclude`, `--note` otherwise). A blank value counts as
  absent.
- **One write.** `apply_act` is the only writer of a finding's state fields and
  its journal. It sets `status` and any given `disposition`, `route` and
  `response`, and appends the turn row, in one `toml_edit` edit that preserves
  comments, unknown keys and sibling findings, and never removes a key.
  `append_finding` writes a new finding together with its `raise` turn.
  `append_review_turn` writes the `conclude` turn. `conclude` sets `concluded =
  true` in the same edit, and reports `already` if the marker was already set.
  Open or answered findings do not block `conclude`.
- **Clearing.** `Act::clears_concluded` is true for `raise` and `reopen`.
  `clear_concluded` flips a `true` marker to `false` in place, in the same edit
  as the clearing act's turn, and leaves an absent or `false` marker untouched.
  A refused act clears nothing.

## Turn protocol

Every mutating act runs inside `turn::with_turn`, the single turn-taking seam
(ADR-007 D-C3, D-C4, D-C4a):

1. Acquire the per-review lock: a `create_new` file at
   `.doctrine/state/review/NNN/lock` with a `pid` and timestamp body, released
   on drop. Contention refuses with `ReviewError::LockContention`, which tells
   the caller to re-run. `review unlock` removes a stale lock left by a hard
   kill.
2. Read and snapshot the authored ledger bytes (`read_authored`).
3. **Entry CAS.** If the runtime baton's `authored_hash` differs from the
   snapshot's sha256, an edit landed before this invocation. The baton is healed
   from the authored ledger, and the act refuses with a re-run instruction. A
   missing baton is cold and proceeds.
4. Static role check: `role` must equal `act.required_role()`, or
   `ReviewError::RoleMismatch`.
5. **Authored first.** The verb's closure runs the per-finding gate (an
   out-of-vocabulary status refuses with `ReviewError::UnknownStatus`, and a
   `can` refusal with `StateMismatch`), then the edit. The counter seed and any
   concluded-clearing ride the same edit. The **pre-write CAS** then re-reads
   the ledger. If the bytes differ from the step-2 snapshot, the act refuses and
   writes nothing. Otherwise the ledger is written atomically.
6. Recompute await and the new hash from the written ledger.
7. **Baton last.** Write `baton.toml` (`awaiting`, `authored_hash`).
8. Release the lock.

The baton is a regenerable cache in runtime state (ADR-007 D-C1, D-C2).
`review status` takes the lock, recomputes the baton from the ledger and
rewrites it. `review prime` takes the lock but neither the baton nor the CAS.

**Locus.** `resolve_review_root` resolves the invoking tree. The runtime subtree
is root-derived, so a review driven from a dispatch coordination tree keeps its
baton there. A root classified as a worker fork is refused, because a fork cannot
co-write the parent's runtime state (IMP-024).

**Roles.** `--as` is a cooperative assertion, not a security boundary (ADR-007).
Omitting it asserts the act's required role. `parse_role` accepts `raiser` and
`responder`, then the ledger's own `raiser` and `responder` labels as aliases.
The labels are fixed at `new`, so `resolve_role` reads them before the lock is
taken. `review new` refuses labels that collide with each other or that name the
other role's canonical token. An unknown token is refused with a message listing
the ledger's labels.

**Counters.** `derive::seed` and `derive::counters` derive the observability
counters from the journal. A ledger's first journalled write, on a ledger with
no `rounds_base` and no turn rows, copies the baton's legacy `rounds` and
`contests` (0 if there is no baton) into `rounds_base` and `contests_base`
(`transition::write_counter_seed`). From then on:

- `rounds = rounds_base + every turn`, finding-level and review-level, unknown
  acts included;
- `contests = contests_base + turns whose act is contest`.

A ledger never journalled reports the baton's values. The baton no longer
increments either counter.

## Derivation rules

`derive::derived_status(findings, concluded) -> (ReviewStatus, Await)` is
evaluated in order:

| condition | status | await |
|---|---|---|
| any finding `open`, `contested`, or out of vocabulary | `active` | `responder` |
| else any finding `answered` | `active` | `raiser` |
| else (every finding terminal, the empty ledger included) and not `concluded` | `active` | `raiser` |
| else (every finding terminal) and `concluded` | `done` | `none` |

So **`done` holds exactly when every finding is terminal and the raiser has
concluded the current pass.** An empty ledger is never vacuously done. Status is
derived on every read (`ReviewDoc::derived`, show, list, status, the catalog
overlay, the baton refresh) and is never latched. Await is a priority summary
for display and handoff, not a gate. The per-finding gate is `can` (ADR-007
D-C4 clarification).

**What `concluded` means.** It says the examination of the current pass is
closed. `conclude` sets it with a basis recording what was examined. Disposing
and verifying may follow a conclude: they resolve findings, and they do not
reopen the examination. `raise` and `reopen` clear it in their own write, so
`done` needs a fresh `conclude` after the last raise or reopen. Earlier
`conclude` turns stay in `[[review.turn]]` as the record of each pass.

**The design run's reading (DEC-138).** `gate::read_pass_facts` returns
`PassFacts { concluded, undisposed_blockers, outstanding, defects }` for a
named RV. The design run admits a `Conducted` review disposition only when the
observed pass is `concluded`, and reads the marker at the moment that
disposition is recorded. A disposition already recorded stands after a late
raise, and the live `reviewing → locked` edge keeps reading only
`undisposed_blockers`, so a late blocker still holds that edge.
`gate::observe_pass` is the gate's posture over the same read: an unparseable
or unreadable ref yields `None`, which the gate treats as refusal, never as
"no blockers". The projection path uses `read_pass_facts` directly and
propagates the error.

The design run mints its own review pass through `review::mint_review`, whose
reserved-id midpoint lets the caller journal the claimed id before any byte is
written. A resumed mint scaffolds into the already-reserved id through
`review::materialise_review_at`. The run side of that bind belongs to SPEC-029.

## Fail-safe reads

A hand-edited ledger can carry a value outside a closed vocabulary. The ledger
never substitutes a known value for it, and never lets it weaken a gate.

- **`Vocab<T>`** is `Known(T)` or `Unknown(raw)`. `Vocab::read` classifies an
  authored string, `as_str` renders the known token or the raw string verbatim,
  and serialisation writes the rendered token, so `--json` and MCP readers see
  what the ledger says. The typed `review::Finding` projection used by `show`,
  the finding index, `--json` and MCP carries `status` and `severity` as
  `Vocab`, with no `Open` or `Major` fallback.
- **Unknown status** reads **non-terminal**. It keeps its review `active`
  (await responder), counts toward every blocker predicate, and admits **no
  act**. The verb refuses with `ReviewError::UnknownStatus`, which names the
  known set and the repair: correct the value in the ledger TOML, after which the
  next turn's entry CAS heals the baton.
- **Unknown severity** gates as a blocker. `gate::gates_as_blocker` is true for
  `blocker` or any out-of-vocabulary severity, and it is the single severity
  classification the three blocker predicates share. `outstanding_by_severity`
  counts it in the `blocker` bucket.
- **Unknown disposition, route, act or role** in a finding or turn is rendered
  verbatim, gates nothing and, for turns, still counts as a turn. These are not
  defects.

**Disclosure (DEC-319, STD-003).** `derive::vocabulary_defects` names each
out-of-vocabulary finding `status` or `severity` as a `VocabDefect { finding,
field, raw, effect }`. The effect is `EFFECT_UNKNOWN_STATUS` ("reading as
non-terminal") or `EFFECT_UNKNOWN_SEVERITY` ("gating as blocker"). Every
surface renders it through `VocabDefect::describe` or `warning_line` (prefix
`WARNING_PREFIX`, always naming the RV), on its own channel:

| surface | channel |
|---|---|
| `review show`, `review status` | a `warning:` line per defect in the text output; a `warnings` array (`review::ReviewWarning`) in JSON and MCP output, absent when there are none |
| `review list` | a `warnings` array across the listed RVs, in id order and never capped. The CLI writes it to stderr so the table cells stay single-valued; `--json` and MCP `review_list` carry it as a field |
| slice close gate | `BlockerRef.reason`, printed beside the refused finding |
| cross-kind catalog | `derived_status_string` returns the status with its defects, and `catalog::scan` pushes one warning `CatalogDiagnostic` per defective RV. Most catalog callers do not surface warning diagnostics yet (ISS-492), so on that path the disclosure is emitted but not yet shown |
| design run | `PassFacts.defects`, which no predicate reads; the design command prints a `warning:` line per defect wherever it reads `PassFacts` |

## Gate predicates

Three pure predicates read the same ledger with deliberately different state
filters. Each is spelled separately so that no shared filter silently picks a
side:

| predicate | asks | holds a finding when |
|---|---|---|
| `doc_unresolved_blockers` | is this review finished, for the target's closure (ADR-007 D-C9b) | it gates as a blocker and is not *known* terminal (so `answered` holds), on a review whose derived status is `active` |
| `undisposed_blockers` | has this pass been disposed of, for a design run | it gates as a blocker and is `open`, `contested` or out of vocabulary |
| `outstanding_by_severity` | what a reader still owes work on | it is not known terminal, bucketed by severity |

**Close gate (D-C9b).** `gate::unresolved_blockers_for(root, subject_ref)` is a
corpus scan: it reads every ledger under the review tree, keeps those whose
`[target].ref` equals the subject (any phase scope matches), and applies
`doc_unresolved_blockers` to each. It builds no reverse index and keeps no
materialised status. The scan is O(number of RVs) per close, and an index is not
built until a measurement shows the scan costs (IMP-479). The slice command
calls it on the closure-seam moves (`audit → reconcile`, `reconcile → done`) and
refuses while any `BlockerRef` is returned, naming each `RV-NNN/F-n` and its
`reason`.

`relation_edges` projects the single `reviews` edge for the relation graph
(ADR-004: authored outbound, reciprocity derived).

## Write surface

The command family is `review new | list | show | raise | dispose | amend |
verify | contest | reopen | withdraw | conclude | status | prime | unlock |
paths` (`review::ReviewCommand`). The MCP server exposes `review_new`,
`review_list`, `review_show`, `review_raise`, `review_dispose`, `review_amend`,
`review_verify`, `review_contest`, `review_reopen`, `review_withdraw`,
`review_conclude`, `review_status` and `review_prime`. Both adapters call the
same `run_*` function per act, so there is one write function per act.

- **`new`** validates the target before any id is claimed, and refuses a
  dangling ref with `ReviewError::DanglingRef`. It renders the ledger with no
  findings and the brief. The default title is `<facet> review of <ref>`.
- **Prose arguments.** In the CLI shell, `ReviewCommand::resolve_prose`
  resolves `--title`, `--detail`, `--response`, `--note` and `--basis` through
  `input::resolve_prose` before any verb runs: `-` reads stdin in full,
  `@path` reads a file relative to the working directory, and anything else is
  literal. A bare `@` is refused. A title read from stdin or a file drops its
  trailing line terminators (`input::resolve_prose_title`); other prose is kept
  verbatim. At most one flag may read stdin per invocation
  (`input::refuse_second_dash`, checked before any read). An empty `--title`,
  `--detail` or `--response` is refused after resolution
  (`input::require_nonempty`), and an empty required note or basis reaches the
  verb's own `NoteRequired` guard. The `@` in a `--target` ref is a phase scope,
  not a file read.
- **MCP** passes JSON strings straight to the verbs with no prose resolution,
  so a literal `-` or `@x` stays literal. Required notes (`review_contest.note`,
  `review_amend.note`, `review_reopen.note`, `review_conclude.basis`) and the
  closed `disposition` and `route` enums match the CLI. A missing required note
  deserialises as empty and refuses with `NoteRequired`.
- **`list --target`** takes the same `REF@PHASE-NN` spelling. A bare ref matches
  every phase scope on that subject, and the phase spelling narrows to one.
- **`show`** renders the derived status, the `reviews` edge, a finding index and
  the brief. `show --json` carries the whole ledger, journals included.
  `status` reports the derived status, await, finding count, rounds, the
  concluded marker when set, and the cache verdict.

## Prime

`review prime` fills a per-review reviewer-context cache,
`.doctrine/state/review/NNN/cache.toml`. It lives in runtime state beside the
baton, and it is regenerable and never authored (ADR-007 D-C10). Its model is
the **target slice's selector path-set**:

- `run_prime` resolves the RV's `[target].ref` as a slice ref and reads that
  slice's declared selectors (every intent).
- A **literal** selector (one with no `*`, `?` or `[`) is kept when it names a
  regular file or is absent: absence hashes as absent, so drift appears the
  moment the path does. A literal naming a directory, a symlink (to anything,
  never followed; checked with `symlink_metadata`) or another special file is
  excluded and listed on a `skipped non-file selector:` line.
- A **glob** expands against the tracked regular blobs (`git ls-files --stage`,
  modes `100644`/`100755`), so symlinks and gitlinks are never hashed.
- Under the per-review lock, the union is hashed with `contentset::compute`, and
  the cache stores the paths and their content hashes.

`review status` reports the cache as `current`, or `stale` with the drifted
paths (changed, removed or added) (`cache_staleness`). Staleness is a signal,
not a gate. Because the key is content hashes, drift in uncommitted or
gitignored files is seen as well as committed drift.

**Degrading.** When the target is not a slice ref, or the slice declares no
selectors, `run_prime` does not fail. Under the lock it removes any
`cache.toml` left by an earlier prime, so `status` reports no cache rather than
a stale `current`. It then returns `ReviewOutput::Primed { tracked_count: 0,
degraded: Some(reason), cleared }`, prints `primed nothing: <reason>` (plus
`removed the previous cache` when `cleared`), and exits 0. The MCP output
carries `degraded` and `cleared`. There is no reviewer-authored prose tier and
no `domain_map`: the cache is the selector model alone.

## Concerns

- **Concurrency is local.** The lock serialises invocations in one tree, and
  the two CAS windows catch out-of-band edits the lock cannot see. Across
  worktrees the backstop is git conflicts on the authored ledger, and
  cross-worktree coordination uses the ADR-006 funnel (ADR-007 D-C7).
- **Hand-edits are expected, not trusted.** Readers classify, never coerce, and
  every classification that changes a gate's answer is disclosed on the
  caller's channel.
- **Close-gate cost** grows with the number of reviews, since each closure-seam
  move reads every ledger. This is accepted until measured.
- **Role assertion is cooperative.** `--as` prevents accidental out-of-turn
  writes, not deliberate ones.

## Hypotheses

- A single-subject ledger with a per-finding act table is enough for every
  review facet. Multi-artefact drift is a different model (ADR-007 D-C11).
- A corpus scan stays cheap enough for the close gate at realistic review
  counts.
- A content-hash cache over the slice's declared selectors captures what a
  reviewer needs re-checked between rounds, without reviewer-authored prose.

## Decisions

- **ADR-007** governs the kind, the baton, the lock and CAS, derived status, the
  two gates and the single-ref subject. This spec is its mechanism.
- **ADR-001**: the engine and command split is enforced by the layering map.
  Nothing in `review_ledger` imports a command-tier module.
- **DEC-138**: a review disposition binds the responder's turn. The design run
  reads `concluded` when a `Conducted` disposition is recorded, and the live
  edge reads `undisposed_blockers`.
- **DEC-233 / DEC-318**: the cross-kind status probe keeps RV's `Unavailable`
  arm, and the catalog overlay supplies RV's derived status through
  `derived_status_string`, until the engine reader calls the derivation
  directly.
- **DEC-319**: an unknown RV vocabulary value warns at the read site, on each
  surface's own channel.
