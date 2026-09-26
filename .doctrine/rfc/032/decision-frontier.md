# RFC-032 companion: the decision frontier

**Role:** companion to `RFC-032` (*Review ledger effectiveness*) and its
[`research.md`](./research.md). The research owns the evidence (findings F1–F13,
candidates C1–C27); the RFC owns the programme. This document takes the design
questions the programme raises and proposes **one opinionated, mutually consistent
set of pivotal decisions** — the point on each question's trade-off curve that
gives the best overall balance of functional fit, complexity, usability and
maintainability.
**Status:** proposal (authority tier 5 — evidence and judgement, not canon). Nothing
here is decided until a slice design locks it or a REV promotes it.
**Compiled:** 2026-09-26 at `edge` `edd986b19`. Code claims re-checked against
`src/review.rs`, `src/reserve.rs`, `src/design_run/{snapshot,attestation}.rs`.
**Revised:** 2026-09-26 after an external design review (codex, `gpt-6-sol`). All
seven points were verified against the code and specs and integrated: the dependency
gate (`D2`/`D14`), locus tiers and RV ownership (`D7`), the reservation namespace
(`D9`), journal limits (`D1`), adoption admission (`D6`), assessment-before-spec
(`D13`), and the `ISS-314` edge (§3). "Dominated" claims without per-axis evidence
are now worded as preferences.

---

## 0. The answer in one page

The RFC frames the programme as nine or so independent fixes in four workstreams.
They are not independent. Most of the friction traces to **three structural
choices**, and fixing those three dissolves a large share of the rest:

1. **The finding has state but no history.** (`D1`) Give every finding an
   append-only turn journal. Contest/verify reasoning becomes durable, `rounds`
   becomes derivable from authored state (over a stored legacy baseline), and
   out-of-band *status* edits become detectable.
2. **The ledger's read model lives in the command tier and has no projection.**
   (`D4`, `D5`) Lift schema + read + derive into an engine-tier ledger module with
   one serialisable view. `show`, `list`, the census, MCP, the design run's gate
   and cross-kind status (`IMP-433`) all read one projection.
3. **Ids are reserved per tree, not per clone.** (`D9`) Claim ids in the git
   common dir, in their own ref namespace. That removes the id-collision class for
   every kind, and it is one precondition for relaxing the locus rule. It does not
   stop two trees *editing* one RV; `D7` answers that separately.

Around those three, the frontier choices are deliberately the *narrow* ones:
narrow status fix (`D2`), adopt-don't-mint for design passes (`D6`), closed-write /
open-read vocabularies (`D8`), no reverse index (`D12`), ADR-007 keeps the *why*
and a new tech spec owns the *how* (`D13`).

**Sequencing consequence.** The RFC orders design-run unification first because
`ISS-314` (the empty-ledger status defect) `needs` `IMP-392`. That edge is
stale: `IMP-392`'s own body records that the one prerequisite `ISS-314` waited on
(the concluded-pass marker) landed on 2026-08-07, and the edge survives only
because `backlog needs` cannot be retracted (`CHR-057`). With that premise gone,
the dependency order runs the other way — **ledger first, consumers after** —
and "hard parts first" still holds, because the hardest decisions are schema
decisions (§3). The edge still makes `ISS-314` read as blocked, so it has to be
retracted before slice 1 starts. §3 step 0 lands `CHR-057` to do that.

| # | decision | resolves |
|---|---|---|
| D1 | append-only per-finding turn journal; state stays stored | C3 · ISS-280 · ISS-485 · F2 |
| D2 | `done ⇔ all terminal ∧ (non-empty ∨ concluded)`; `complete ⇔ all terminal ∧ concluded` gates dependencies; conclude carries a basis | C1 · ISS-314 · ISS-366 · ISS-322 trap |
| D3 | optional `anchor` on a finding: opaque `(section, fingerprint)` | C9 (IMP-392 remainder) |
| D4 | engine-tier ledger module; `review.rs` split | C24 · IMP-068 · IMP-433 |
| D5 | one read projection: `show` renders the finding index by default | C6 · C7 · C8 · F1 · F10 |
| D6 | design run binds a pass: mint by default, adopt by name under admission rules | C10 · ISS-322 · C11 · C12 (part) |
| D7 | three locus tiers (read / runtime / authored) at the verb-family boundary; git merge is the declared multi-tree backstop | C2 · C17 · F5 · ISS-484 · IMP-240 |
| D8 | roles closed + participant aliases; disposition closed-write; optional closed `route` | C13 · C14 · F4 |
| D9 | clone-wide id reservation in a separate local ref namespace; `reseat` reads leniently | C16 · ISS-279 · ISS-277 · F12 |
| D10 | every prose argument takes `-`/`@file`; MCP is the preferred write path | C4 · IMP-377 · ISS-486 · F3 |
| D11 | ADR-007 D-C10 amended to the lived selector model; `prime` degrades, never fails | C19 · C25 · C27 · IMP-478 · F7 |
| D12 | no reverse index; scan stays | C20 · IMP-479 |
| D13 | leading proposal: a tech spec owns the RV mechanism (boundary fixed by `IMP-481` first); ADR-007 keeps the decision; doc drift is checked | C18 · C22 · IMP-481 · F13 |
| D14 | `RV` admitted as a relation target once D4 lands; gates on `complete`, not `done` | C21 · IMP-480 · F9 |
| D15 | fail-safe reads for every closed vocabulary | C5 · CHR-001 |

---

## 1. Method: what "frontier" means here

Four axes, scored per option: **fit** (does it close the observed failures and
the ones the evidence predicts), **complexity** (new concepts, new code paths,
migration), **usability** (agent token cost, error surface, number of steps), and
**maintainability** (drift resistance, test surface, coupling).

An option is *dominated* when another is at least as good on every axis and
better on one. The word is used only where the option tables show that
comparison. Elsewhere the rejections are judgements and are worded as
preferences. This document picks the option with the best *combined* position,
and says what it gives up. Where
the frontier genuinely forks on a value the user holds (not an engineering fact),
it says so in §5.

A standing bias, per the repo's own principles: prefer options that **remove a
concept** over options that add a check for it.

---

## 2. The decisions

### D1 — Findings carry an append-only turn journal; current state stays stored

**Question.** Where does the reasoning of `contest` / `verify` / `withdraw` land,
and how is a stale response or an out-of-band verify corrected? (`ISS-280`,
`ISS-485`; research F2, A2.)

**Options.**

| | shape | fit | complexity | usability | maintainability |
|---|---|---|---|---|---|
| a | add `contest_reason` / `verify_note` fields, overwritable | partial — `ISS-280`'s RV-346 evidence (F-28 contested twice) shows a single slot loses history | lowest | good | poor — one field per act, grows |
| b | **turn journal per finding; `status`/`disposition`/`response` stay stored as current state** | full | moderate; no migration | good | good |
| c | full event sourcing: turns only, state folded on read | full | high — fold on every read path, legacy shim or migration of ~366 ledgers | same as b | best in theory; more code on the hot path |

**Choice: (b).**

```toml
[[finding]]
id = "F-3"
status = "answered"          # current state — what gates and renders read
severity = "major"
title = "…"
detail = "…"
disposition = "fix-now"
response = "…"               # the current account

[[finding.turn]]             # append-only history; one row per transition
round = 4
act = "contest"              # raise | dispose | contest | verify | withdraw | amend | reopen
role = "raiser"
note = "the repair is partial: …"   # required for contest/reopen/amend; optional otherwise
```

Rules:

- Every transition verb appends exactly one turn in the same CAS-guarded write
  that moves `status`. The prose that today goes to the ephemeral `--note` goes
  to the turn. **The ephemeral note concept is retired** — one note per act,
  durable.
- A `dispose` turn records the response it gave, so a later re-dispose does not
  erase what a contest was arguing against. That is a deliberate redundancy
  between the latest dispose turn and `response`: state and history, written by
  one function in one write, the way a working tree and a log coexist. Accepting
  it is what keeps (b) off (c)'s read-path cost.
- Two new acts close `ISS-485`: `amend` (responder; `answered → answered`, new
  response, note required) and `reopen` (raiser; `verified → contested`, note
  required). `withdraw` stays terminal.
- Legacy findings simply have an empty journal. No migration. History from before
  the journal existed is **not** reconstructed. It never reached authored state.
- **Legacy round baseline.** The first journalled write to a ledger that has no
  turns copies the baton's current `rounds` into an authored `[review]
  rounds_base` (0 if the baton is gone). From then on, `rounds = max(rounds_base,
  max turn round)`.

**What falls out.**

- After that first journalled write, `rounds` is derivable from authored state,
  and the baton's durable content (rounds, await) is fully derivable too. The
  baton lock and the per-invocation CAS stay runtime mechanism. This is what `D7`
  relies on. A legacy ledger that has not yet had a journalled write still depends
  on its baton for `rounds`. The only loss there is a cosmetic counter, which the
  baseline handles.
- **What the doctor check can detect, exactly:** for a finding with a non-empty
  journal, stored `status` must equal the status its last turn implies, and the
  last turn's role must be the one the act allows. That catches hand-set statuses
  (the `01a0d7f7` responder-authored `verified`) and pre-disposed arrivals (the
  `01a0d241` case). It does **not** catch a hand edit to `response`, `detail` or
  `severity` that leaves `status` alone, or a hand-written turn row that looks
  valid. Catching those would need content hashes per turn, which costs more
  than the incidents it would catch. Detection, not prevention — the honest
  ceiling given `--as` is cooperative (ADR-007 Negative).
- The auditor's question "why did rev 2 become rev 3" has an answer in the
  committed file.

**Gives up.** Ledger files grow (~one short table per transition; RV-346's 121
rounds would add roughly that many rows). Acceptable: the file is the audit
record, and the read surface (`D5`) does not render turns by default.

---

### D2 — Status: `done ⇔ all findings terminal ∧ (non-empty ∨ concluded)`

**Question.** What does an empty ledger read as, and how does a clean pass differ
from an untouched one? (`ISS-314`, `ISS-366`, `IMP-098`; research F6, A7.)

**Options.**

| | rule | fit | cost |
|---|---|---|---|
| a | today: empty ⇒ `done` | fails D-C8 and `ISS-366` | — |
| b | ADR-007 as written: empty ⇒ `active/raiser`, always | reintroduces `IMP-098`'s token-nit round for clean audits | trivial |
| c | **narrow: empty ∧ ¬concluded ⇒ `active/raiser`; empty ∧ concluded ⇒ `done`; non-empty unchanged** | closes both observed failure shapes | trivial; uses the existing marker |
| d | uniform: `done` requires `concluded` for every ledger | most coherent (the "one finding, more coming" case is the same ambiguity) | flips ~all legacy all-terminal ledgers to `active`, or needs a backfill that asserts events that never happened; adds a step to every review |

**Choice: (c)**, with one addition: **`conclude` takes a required `--basis`**
(what the pass examined), stored as a review-level turn
(`[[review.turn]] act = "conclude"`), reusing D1's shape.

**Plus a second predicate for anything that gates on a review:**

```
complete ⇔ all findings terminal ∧ concluded
```

`done` is what a reader sees. `complete` is what a *dependency* reads. Under (c),
a non-empty review whose findings are all terminal reads `done` while the raiser
may still be working. That is harmless as a display, but wrong as a gate: once
`D14` makes RV status reachable across kinds, an item that `needs` an RV would
become actionable in the middle of a pass. So every cross-kind gate
(`status_class`, actionability, `D14`) reads `complete`. D-C9b (the close gate)
is unchanged — it reads blockers, not status.

Why not (d): every observed incident is the *empty* case, and (d)'s price is a
semantic flip across the whole corpus. The one place where the non-empty
ambiguity causes harm, dependency gating, is closed by `complete` without
touching any existing ledger's displayed status. No existing edge is affected,
because `complete` is only read by relations that `D14` introduces. The skills
should still make `conclude` the normal last move of every pass.

**What it costs:** two status words where (d) has one. That trade is listed as a
genuine fork in §5.

Why the basis: it defuses the `ISS-322` laundering trap without parsing prose.
Concluding a zero-finding ledger remains legal (a clean pass is a real result),
but it now leaves a durable, attributable statement of what was looked at — the
same *authority and visibility, not prohibition* fence `DEC-138` chose. The gate
does not read the basis; a human or auditor does. Refusing a zero-round
conclude, `ISS-322`'s alternative, is worse on fit: it would also refuse the
honest clean pass.

**Consequences.** The two tests pinning the violating value
(`show_renders_empty_ledger_done_and_the_edge`,
`list_renders_empty_ledger_done_and_the_edge`) flip. D-C9b is unaffected — an
empty ledger has no blocker either way. ADR-007 D-C8 is amended by REV to state
(c), the `complete` predicate, and the conclude marker's role in both.

---

### D3 — The section anchor is opaque to the review kind

**Question.** How does an RV finding carry a design-document section reference
and the fingerprint it was raised against, without the review module learning
about the design run? (`IMP-392`, `DEC-125`'s named "hardest" complication.)

**Choice.** An optional table on the finding:

```toml
[finding.anchor]
section = "sec-3"
fingerprint = "b3:…"     # the section digest at raise time
```

- `review` stores and renders it; it **never interprets it**. Staleness ("the
  section moved since this was raised") is computed by the design run, which owns
  fingerprints, through the existing one-way `design`-shell → `review`-query
  dependency (ADR-001; `read_pass_facts` is the precedent). It **warns**, never
  blocks, per `DEC-126`'s derived-warning precedent.
- The anchor is written at `raise` (raiser-owned, fixed, like `detail`), supplied
  by the caller — in a design run, the envelope prints the current fingerprint
  per section so the raiser can pass it.
- It is **not** generalised to code loci (`path:line`) now. The table is named
  `anchor` so a later `path` key fits without a rename; nothing is built for it.

Rejected: a review-side fingerprint concept (couples review to design-run
digests — ADR-001 direction violation); a free-text `section:` token in `detail`
(the prose-loader risk `DEC-138` already refused).

---

### D4 — An engine-tier ledger module; `review.rs` splits along the tier line

**Question.** Where do the schema, reader and derived status live? (`IMP-433`,
`IMP-068`, `CHR-001`.) `review.rs` is 5,773 lines in the command tier; nothing
below it can ask an RV its status, so `DEC-233` had to return `Unavailable` for
RV targets.

**Choice.** Split once, along the layering line, as the first act of the
programme:

- `review/ledger.rs` (engine tier, pure where possible): the authored schema
  (finding, turn, anchor, meta), the lenient reader, `derived_status`, the
  transition graph, the read projection (`D5`), and the blocker predicates
  (`doc_unresolved_blockers`, `undisposed_blockers`, `outstanding_by_severity`).
- `review/` command modules: the verbs, `with_turn`, baton/lock, prime, render.

This is a behaviour-preserving refactor gated by the existing suites, and it
carries `IMP-029` (the missing e2e golden) as its safety net — write the golden
*first*, then move code. It is the enabler for `D5`, `D14`, `IMP-433` and the
census; doing it later means doing those against a module that is about to move.

---

### D5 — One read projection; `show` leads with the finding index

**Question.** What does reading a review cost, and do CLI and MCP agree?
(F1 — nine observations; F10; `IMP-475`–`IMP-477`.)

**Choice.** One serialisable `ReviewView` built in the ledger module; every
surface is a render of it.

- `review show RV-N` (default): header (status, await, facet, target, rounds,
  concluded), then **one line per finding** — id, severity, status, disposition,
  anchor section, title. The payload a reader came for, at the ~200-token cost
  obs `019fac2f` asked for.
- `review show RV-N --finding F-n`: that finding's detail, response and turn
  journal. `--full`: brief, synthesis, every finding in full.
- Filters shared by show and the census: `--status`, `--severity`, `--open`
  (shorthand for non-terminal).
- `review list --target <ref>` (+ `--facet`).
- **Census** (`IMP-477`): `doctrine review findings` — one row per finding
  across the corpus (review id, facet, target, finding columns), standard
  `listing` machinery, `--format json`. No aggregation verb: counts are a `jq`
  one-liner over rows, and a bespoke aggregator is the thing that drifts.
- **JSON is the projection, not the TOML.** CLI `--json` and the MCP tool emit the
  same shape with `findings` (plural). `review.finding` (the raw array-of-tables
  key) stops leaking. This is a breaking change to CLI JSON; pre-1.0 and the MCP
  shape is the one agents already use, so it is the right side to break.

Rejected: a `--findings` opt-in flag (keeps the default wrong — the default is
what nine observers hit); rendering findings in the brief (mixes raiser seed prose
with ledger state).

**Quick-win lane.** The finding index and `list --target` render *existing* fields
and need no schema change. They can ship as a small backlog item before the
programme's first slice, and `D1`/`D3` columns join later. Given nine recorded
incidents, this is the cheapest friction relief in the programme.

---

### D6 — A design run binds its pass: mint by default, adopt by name

**Question.** How does the run relate to its RV? (`ISS-322`, `ISS-476`,
`ISS-452`, `IMP-392` remainder; research F8, A5.)

**Choice.**

- On entry to `reviewing`, the run **binds** a pass. With no argument it mints, as
  today. With `--review RV-N` (or the equivalent declaration) it **adopts** an
  existing RV. The mint remains the ordinary path; adoption is the route the
  `reviewing` obligation already advertises for an external reviewer.
- **Admission rules for adoption.** Each refusal names the rule it failed. An
  RV is admitted iff:
  1. it targets this slice, has facet `design`, and is readable in this tree;
  2. it is **not concluded** — a concluded RV is a finished pass over some
     earlier state of the design, and adopting it would let a stale review pass
     the gate;
  3. it is **not bound** to any other pass, in this run or any other. The binding
     is recorded on `ReviewPass` and in the change row, so the check is a lookup
     that needs no new store;
  4. every finding that carries an `anchor` names a section that exists in the
     current design. Anchors whose fingerprint no longer matches the section are
     not refused. `D3`'s staleness warning flags them, the same as it does for a
     minted pass.
- Adoption changes SPEC-029's run contract, which today says the run mints its
  pass. The slice that lands `D6` revises SPEC-029 for the bind step, the
  admission rules, and the change row.
- A pass is still one RV; re-entry binds a new pass (mint or adopt). Coverage stays
  on `ReviewPass.covered`, unchanged. Binding *several* RVs to one pass is rejected:
  it makes `Conducted`'s predicate a union over ledgers for no observed benefit;
  other reviews of the slice stay visible through `list --target`.
- The envelope names the bound pass RV at every stage of `reviewing` (the
  remaining half of `ISS-476`), and `ForeignPass` refusals name both the adopt
  route and the bound RV.
- Binding emits a change row (`ISS-452`).
- The runtime `Finding` and its `fnd-` declaration route are **deleted** —
  `IMP-392` records the store is write-only since `SL-244` PHASE-05, so this is a
  deletion, not a migration.
- `ISS-462` (the undeclarable `ReviewPolicy` labels) rides along as a const-assert;
  `ISS-310` / `ISS-359` stay separate — they are attestation questions, not
  ledger ones.

With `D2`'s basis, the `ISS-322` sequence (conclude an empty run-minted RV, point
the disposition at a different ledger in prose) has no reason to exist.

---

### D7 — Three locus tiers, applied once; git merge is the declared multi-tree backstop

**Question.** Where may review verbs run? (`ISS-484`, `IMP-240`, `IMP-190`,
`IMP-024`; research F5, A3.)

Today the fork refusal (`resolve_review_root`, `src/review.rs:2358`) exists
because a fork cannot co-write the parent's baton. Read verbs skip it: `run_show`
resolves its root directly (`src/review.rs:1878`). Under `D1` (once a ledger has
had a journalled write) the baton's durable content is derivable, so a review
worked in one tree is a single-tree review wherever that tree is.

**Choice — three tiers, one table, evaluated once in the verb-family dispatcher:**

| tier | verbs | admitted where |
|---|---|---|
| read | `show`, `list`, `findings` (census), `status` | any tree the root resolves in, including a confined worker |
| runtime | `prime` (reviewer cache in `.doctrine/state/`) | any tree with a writable state tier |
| authored | `new`, `raise`, `dispose`, `contest`, `verify`, `withdraw`, `amend`, `reopen`, `conclude` | any tree that can durably write `.doctrine/review/`. Refused in a confined dispatch worker (the `DOCTRINE_WORKER` marker / read-only authored tier) |

- `new` goes through the authored tier *before* it allocates an id. That closes
  `ISS-484`: no id is allocated in a tree that then refuses.
- Primary, coordination and solo `/worktree` trees are admitted for authored
  writes, which is what `AGENTS.md`'s "audit/close on a worktree" instruction
  already assumes.
- `IMP-024`'s large-review funnel stays deferred; it is about fanning a raiser,
  not about where a single-tree review may live.

**Two trees editing one RV.** Unique ids (`D9`) do not stop this. The
choice is to **declare git merge the backstop** rather than add an ownership
mechanism:

- two trees that move *different* findings merge cleanly, and that result is
  correct, because findings are independent;
- two trees that move the *same* finding both rewrite its `status` line, and git
  raises a textual conflict that someone has to resolve;
- after the merge, `D1`'s doctor check confirms that each finding's `status`
  agrees with its last turn.

Rejected: recording a home tree or branch on the RV and refusing writes
elsewhere. Trees are ephemeral and branch names do not survive landing, so the
recorded owner would go stale in the normal lifecycle. ADR-007 D-C7's
single-tree rule is amended by REV from "one tree" to "one writer at a time per
finding, with merge as the backstop".

**Dependency.** Admitting more trees widens the id-collision window, so `D7`
ships **with or after** `D9`, never before.

---

### D8 — Vocabularies: closed roles with aliases; disposition closed-write, open-read; an optional closed `route`

**Question.** How much vocabulary does the ledger enforce? (F4; `IMP-336`;
RFC-026 E1's 61 disposition values; RFC-026 `P10`.)

**Choice.**

- `--as` stays the closed `{raiser, responder}`; the labels declared at `review
  new` are accepted as aliases, and `--help` names the legal values (`IMP-336`).
- `disposition` becomes **closed on write** (the five documented values), **open
  on read** — legacy values render verbatim and never fail a read (`D15` governs
  what they may *gate*). The refusal names the set.
- An optional closed `route` field (`review | demonstrate | probe | control |
  owner-fix`) replaces the `route:<route>` prose token. **Optional**, not
  required-for-severe, until RFC-026's `P10` trial reports; the trial then decides
  whether any facet requires it. Structuring it now means the trial is counted by
  the census instead of by regex.
- `follow-up` dispositions cite what they spawned through `D14`'s relation, not a
  new finding field.
- `@PHASE-NN` target spelling (C15): accept it on input, since the kind displays
  it. One parser, trivial.

---

### D9 — Ids are reserved clone-wide, in a local ref namespace of their own

**Question.** How do two trees of one clone stop minting the same id? (`ISS-279`,
`ISS-277`, obs `01a0bc8d`; research F12, A6.)

The engine already has the right mechanism for the wrong scope: `reach = shared`
claims `refs/doctrine/reservation/<PREFIX>/<id>` by a zero-oid create CAS on a
remote (`src/reserve.rs`, `GitRef`). This repo pins `reach = "local"`
(`.doctrine/doctrine.toml`), whose claim is a per-tree `mkdir`. Linked worktrees
share one ref store.

**Choice.** Make the **local** claim clone-wide: a zero-oid `update-ref` CAS in the
common dir, scanned alongside the local directories. No remote, no network.
Confined workers never mint (sole-writer), so their read-only `.git` is not a
constraint. This fixes the class for **every** kind, not just RV.

**Namespace.** Local claims use their **own** prefix,
`refs/doctrine/reservation-local/<PREFIX>/<id>`. They must not share the
shared-reach prefix. The shared backend fetches that prefix with a force
refspec (`+refs/doctrine/reservation/*:refs/doctrine/reservation/*`,
`src/reserve.rs:117`), so a local claim stored there would be silently
overwritten or deleted by the next shared-reach fetch. A separate prefix keeps
the two reaches from interfering:

- **Allocation scans both namespaces** whichever reach is configured. A clone
  that moves from `local` to `shared` keeps seeing its own local claims, so it
  cannot re-mint them.
- **Local claims are not pushed** when the clone moves to shared. Other clones
  never saw them, and the ids already exist as authored entities, which every
  clone's allocation scans after a pull. So the switch is not retroactive, the
  same as a `local → shared` change today.
- **Semantics change.** `reach = local` changes meaning from "this tree" to "this
  clone". PRD-005 (the reduced-reach wording) and SPEC-008 (the local-only
  degradation clause) are revised in the slice that lands `D9`. This is a
  compatibility change to the reservation contract. It reuses the scan shape
  but is more than a second arm of `GitRef`.

Alongside: `reseat` reads the alias slug through the lenient reader (`SL-151`'s
fix, applied to the path it missed — `ISS-277`), so the collisions that already
exist can be repaired by verb.

The research cites a preflight note that `DOCTRINE_TRUNK_REF` is "a zero-cost fix
already in the engine". At `edd986b19` that variable is read only by dispatch
trunk resolution (`dispatch_config.rs`, `worktree/coordinate.rs`), not by
`reserve.rs`; treat the claim as unsubstantiated.

---

### D10 — Prose arguments are file/stdin-capable; MCP is the preferred write path

**Question.** How are shell-expansion incidents prevented? (obs `019fc0bc` — five
live API keys spliced into a committed ledger; `019fd757`; `IMP-377`, `ISS-486`.)

The expansion happens in the caller's shell; Doctrine cannot see it, only its
result. So the frontier is about giving callers a route with no shell in it.

**Choice.**

- Every prose argument on every review verb (`--detail`, `--response`, `--note`,
  `--basis`, `--title`) accepts `-` (stdin) and `@path`. One shared arg type, not
  per-verb code.
- The skills and `review-ledger.md` route agents to the MCP tools for writes (JSON
  arguments, no shell) and, for CLI use, to a **quoted** heredoc
  (`<<'EOF'`) — the one idiom that disables expansion.
- Secret detection is **not** built into the review kind. It is a repository-wide
  concern (a pre-commit scanner), and a review-local heuristic would be a partial
  control that invites being relied on. Note it as a separate backlog item.

---

### D11 — ADR-007 D-C10 is amended to the model the code already implements

**Question.** Re-affirm the reviewer-authored `domain_map` warm-cache, or bless
the lived selector model? (`IMP-478`, `IMP-259`, `ISS-059`; research F7, A4.)

**Choice.** Amend. D-C10 becomes: the reviewer-context cache is the target's
**declared path-set** (slice selectors), content-hash keyed; the reviewer-authored
prose tier is retired, on `RFC-004`'s finding that it had zero readers. The
worktree-aware staleness deferral is closed as moot: the cache is per-tree
runtime state keyed on content the tree can read.

- `prime` on a target without a path-set (non-slice subject, zero selectors)
  **degrades**: it primes nothing and says so (STD-003 — disclosed, not silent),
  instead of failing the review. (`IMP-259`.)
- The literal-selector arm gets the same non-file filter the glob arm has
  (`ISS-059`).
- `IMP-025` (promote the content-set to a shared primitive) is left to its own
  trigger; nothing here creates the second consumer it waits for.

Re-affirming D-C10 and building the `domain_map` reader is not preferred: it
restores an authoring tax whose value the one study of it (`RFC-004`) could not
find.

---

### D12 — No reverse index for the close gate

**Question.** Does D-C9b's corpus scan need an index? (`IMP-479`; ADR-007 R2.)

**Choice.** No. The scan is O(#RV) file reads once per `/close`; at ~366 ledgers
it is not a measurable cost, and ADR-004's outbound-only rule makes an index a
second store to keep coherent. `D4` puts the scan beside the other predicates,
so if it ever matters, the index has one place to land. Close `IMP-479` as
*not needed until measured*; do not invent a tripwire.

---

### D13 — A tech spec owns the mechanism; ADR-007 keeps the decision; the shipped doc is checked

**Question.** Who owns the RV kind, and how does the protocol doc stop drifting?
(`IMP-481`, `CHR-079`; research F13, A8.)

**Order: assess, then author.** `IMP-481` (a `/spec-coverage-assessment`) runs
**before** slice 1's design locks, not in slice 4. Adjacent pieces are already
owned elsewhere: SPEC-013, SPEC-008 (reservation, which `D9` touches) and
SPEC-029 (the design run's pass, which `D6` touches). The assessment decides the
new spec's scope and parent, and which pieces belong in an existing spec instead.

**Leading proposal** (subject to the assessment):

- A **tech spec** owns the ledger mechanism: schema (finding, turn, anchor,
  route), the transition graph, `done` and `complete` (`D2`), the read
  projection, the locus tiers, conclude semantics. It is authored **in the first
  slice** (`D1`/`D2`/`D4` land there) and extended by each later slice — the
  RFC's "each slice leaves spec coverage" intent, given a concrete home.
  Reservation (`D9`) and the design-run binding (`D6`) stay in SPEC-008 and
  SPEC-029, and are revised there.
- **No new PRD.** The product intent ("adversarial review at every lifecycle
  stage, with teeth") is one paragraph of ADR-007's context; if the
  assessment finds an existing PRD that should carry it, add it there.
  A PRD for a single mechanism kind is overhead.
- ADR-007 keeps what it is good at — *why* a first-class kind, why turn-based,
  why single-tree — and its drifted clauses (D-C8, D-C10, D-C5's field ownership,
  D-C1/D-C7's fork clarification) are amended by REV to state the decision and
  cite the spec for mechanism. That removes the duplicated-mechanism surface that
  drifted.
- `install/review-ledger.md` is refreshed once (`CHR-079`), then **checked**: every
  `doctrine review <verb>` and `--flag` the doc names must exist in the clap
  surface. Build it as a doctor check alongside the existing `ProseCite` scanner
  (same category family, `src/doctor_checks.rs`) and point it at every shipped
  install doc, not just this one — the drift class is not review-specific.

---

### D14 — `RV` becomes a relation target once its status is reachable

**Question.** Can a backlog item cite the review that raised it? (`IMP-480`,
`IMP-433`; F9.)

**Choice.** Yes, after `D4`: admit `RV` as a `references` target (role
`originates_from` included), and let `partition::status_class` read its status
through the engine-tier ledger module, retiring `DEC-233`'s `Unavailable` arm
for RV. **The gate reads `complete`, not `done`** (`D2`): an RV counts as settled
for a dependant only once its raiser has concluded and every finding is
terminal. Otherwise a `needs` edge on an RV would become actionable in the
middle of a pass. Finding-granular citation (`RV-N#F-3`) is **not** introduced; prose
can name the finding, the edge names the review. Admitting `REC` is the same
change and should ride it.

---

### D15 — Every closed vocabulary reads fail-safe

**Question.** What does a hand-corrupted enum value do? (`CHR-001`, RV-026 F-2.)

**Choice.** One rule for the ledger reader: an out-of-vocabulary `severity` gates
as `blocker`; an out-of-vocabulary `status` reads as non-terminal and emits a
doctor finding; an out-of-vocabulary `disposition` renders verbatim (`D8`). No
silent coercion to `Open`. The baton-note write moves inside the lock/CAS (the
same `CHR-001` item), which `D1` makes natural — the note *is* the authored turn.

---

## 3. Sequencing

Re-cut from the RFC's four slices. The order follows dependencies, not
workstream labels, and still puts the most consequential decisions first.

| order | slice | decisions | why here |
|---|---|---|---|
| 0a | *unblock (backlog item)* | land `CHR-057` (a verb to retract a `needs` edge), then retract `ISS-314 needs IMP-392` | the stale edge makes slice 1's status fix read as blocked under ADR-017 gating; retract it through a verb, not a hand edit |
| 0b | *assessment* | `IMP-481` via `/spec-coverage-assessment` | fixes the tech spec's scope and parent before slice 1 authors it (`D13`) |
| 0c | *quick win (backlog item)* | D5's finding index + `list --target` over existing fields | nine incidents; no schema dependency. Build it as a render over a view struct, not bespoke formatting, so D4 moves it without rewriting it |
| 1 | **Ledger v2** | D4 (split, with `IMP-029` golden first), D1, D2 (incl. `complete`), D15, D10, D8 (write side), authors the tech spec | every later slice reads this schema; the hard, hard-to-reverse choices live here |
| 2 | **Read surface** | D5 remainder (projection, census, JSON unification), D14 | needs D4's module and D1/D3's columns |
| 3 | **Design-run binding** | D3, D6 (adopt, `Finding` deletion, change row, envelope naming, `ISS-462`) | a *consumer* of the ledger; builds on D1–D3 |
| 4 | **Identity & locus** | D9, then D7 | D7 is unsafe before D9 |
| — | rides whichever slice touches it | D11 (with any `prime` change, else slice 1), D12 (close `IMP-479`), D13's doc check (slice 1 or 2) | governance reconciliation, not work of its own |

Slices 3 and 4 are independent of each other and can run in parallel. The 0-steps
are small and independent; 0a and 0b must land before slice 1's design locks.

**Governance route** (unchanged from the RFC, made concrete): slice 1's REV
amends ADR-007 D-C5 (turns, amend/reopen), D-C8 (`D2`, including `complete`)
and, if `prime` is touched, D-C10 (`D11`). Slice 3 revises SPEC-029 (bind and
adoption, `D6`). Slice 4's REV amends ADR-007 D-C1/D-C7 (`D7`'s tiers and the
merge backstop) and revises PRD-005 / SPEC-008 (`reach = local` means "this
clone", `D9`). The tech spec is authored in-slice and grows per slice.

**Answers to the RFC's open questions.**

- *Slice 1 boundary* — the design-run cluster is not slice 1 at all. Slice 1 is
  the ledger; the design-run cluster (C10–C12) lands whole in slice 3, except
  `ISS-310` / `ISS-359`, which are attestation questions and stay separate.
- *Does a spec own the RV kind?* — leading proposal: yes, a tech spec for the
  ledger mechanism, authored in slice 1 (`D13`). Its scope and parent are fixed by
  `IMP-481`, which moves forward to step 0b.
- *Close-gate cost* — no index (`D12`).

---

## 4. What this set gives up, stated once

- **Ledger size** grows with the turn journal (`D1`). Chosen over losing the
  reasoning that ADR-007 exists to keep.
- **CLI JSON breaks** (`D5`). Chosen over keeping two shapes forever.
- **Two status words** (`done` for display, `complete` for gates) (`D2`). Chosen
  over a corpus-wide semantic flip. The non-empty premature-`done` ambiguity
  stays in the display, where it has no recorded failure. It is closed for every
  gate.
- **Out-of-band edits are detected, not prevented, and only status edits**
  (`D1`). Hand edits to prose fields that leave status alone go undetected.
  Nothing cooperative can do better at an acceptable cost.
- **Pre-journal history is not recovered** (`D1`). It never reached authored
  state.
- **Two trees can edit one RV** (`D7`). Git merge is the backstop, and conflicts
  on the same finding surface as text conflicts. No ownership field.
- **`reach = local` changes meaning** (`D9`), from "this tree" to "this clone",
  with a spec revision.
- **No secret scanning in review** (`D10`). A repository-wide control belongs
  elsewhere.
- **The route field stays optional** (`D8`) until RFC-026's trial speaks.

---

## 5. Where the frontier genuinely forks

Two choices depend on a preference rather than an engineering fact; the
recommendation is stated, and each is cheap to revisit.

1. **`D2` narrow + `complete` vs uniform.** Both are safe for gating. The choice
   is two status words and no migration, or one word plus a migration. If the
   user values a single rule (`done` always needs `conclude`, and `complete`
   collapses into it), take option (d) and backfill `concluded = true` for legacy
   all-terminal ledgers with a disclosed migration note. Recommendation: narrow +
   `complete`. The backfill would record conclusions that never happened.
2. **`D5` default render.** If default `show` output must stay minimal for some
   consumer, invert to a `--findings` opt-in. Recommendation: findings by default
   — nine observers hit the default, and none asked for less.

---

## 6. Backlog deltas implied (not filed by this document)

- New: amend/reopen acts (partly `ISS-485`), conclude `--basis`, the `complete`
  predicate for cross-kind gating, the review locus-tier table, the install-doc
  flag conformance doctor check, clone-wide local reservation (own namespace,
  PRD-005/SPEC-008 revision), design-run adoption admission rules (SPEC-029
  revision), repo-wide secret scanning, the quick-win finding index (split out of
  `IMP-475`).
- Promote: `CHR-057` to step 0a — it now blocks the programme, not just tidies
  it. `IMP-481` to step 0b.
- Narrow: `IMP-392` (to D3 + D6).
- Close on landing: `ISS-314`, `ISS-366` (duplicate), `IMP-479` (not needed),
  `IMP-478` (by REV).
- Retract: `ISS-314 needs IMP-392`, by `CHR-057`'s verb (step 0a).

[RFC-032 decision-frontier]: `edge` `edd986b19`
