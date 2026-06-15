# SL-069 design

## 1. Current vs target behaviour

**Current.** Doctrine ships 14 orientation memories (ADR-002 class: `repo=""`,
`anchor_kind=none`, scoped, evergreen) embedded in the binary. These cover
doctrine's internal machinery — lifecycle, storage, routing, conventions, TDD —
but zero client operational capabilities: installation, boot snapshot, backlog,
ADR authoring, specs, review, requirements, etc. The boot snapshot's Memory
section renders all 14 in full, and `boot --check` doesn't flag empty governance
sections. The existing corpus is last-author-touched 2026-06-06 (SL-018
PHASE-05/06) and has drifted against the ~30 slices shipped since.

**Target.** Add 13 new shipped memories covering Tiers 1–3 capability gaps (review
deferred to post-SL-068), trim the boot snapshot Memory section to
signpost-only, add `boot --check` warnings for empty governance sections,
and refresh the existing corpus for post-SL-018 reality. The shipped corpus
grows to 27 memories (13 new + 14 existing) forming a cohesive client onboarding
surface — navigable via `find`/`retrieve`, with the boot snapshot pointing at
signposts for orientation.

## 2. New shipped memories

### 2.1 Authoring principles

- **Author fresh, don't promote verbatim.** Non-shipped memories with
  client-relevant lessons (research.md §2) are synthesised into new shipped
  memories with client-appropriate framing and scope. Origin memories cited for
  provenance.
- **Signposts navigate, concepts explain.** New memories use `signpost`
  (navigational pointers to capabilities, CLI verbs, reference docs) and
  `concept` (durable mental models — reading entities, boot snapshot). No
  `pattern`/`fact` for new shipped content.
- **Scoped for retrieval relevance.** Each memory carries scope paths and
  commands matching the capability it orients.
- **Cross-reference siblings.** New memories link to existing shipped memories
  and to each other, building a connected graph.
- **Point, don't restate.** Where `using-doctrine.md` or `glossary.md` already
  cover ground, point there. Never reproduce `doctrine --help` flag tables.
- **Honest about implementation state.** Where capability is in flight or not
  widely adopted, say so (e.g. VT verification shipped but not dogfooded).

### 2.2 Candidate list (13 new, 27 total shipped)

| # | key (provisional) | type | scopes | notes |
|---|-------------------|------|--------|-------|
| 1 | `mem.signpost.doctrine.install` | signpost | `doctrine install`, `install/` | What install does, what ships, idempotency |
| 2 | `mem.concept.doctrine.boot-snapshot` | concept | `.doctrine/state/boot.md`, `doctrine boot`, `.doctrine/governance.md` | Short: what it is, high-level mechanism, how to influence (ADRs, policies, memories), `boot --check`, freshen-now ritual |
| 3 | `mem.concept.doctrine.reading-entities` | concept | `.doctrine/`, `doctrine <kind> show` | The worked failure lesson: read via `show`, never judge from one tier. Cites `mem.system.governance.ship-usage-guidance-to-clients`. |
| 4 | `mem.signpost.doctrine.reference-docs` | signpost | `using-doctrine.md`, `glossary.md` | Points to the two shipped reference docs; what each covers, when to read |
| 5 | `mem.signpost.doctrine.relating-entities` | signpost | `doctrine link`, `doctrine inspect`, `doctrine unlink` | Honest: what's implemented via `link` vs where hand-editing is required. Points to CLI. |
| 6 | `mem.signpost.doctrine.recording-memories` | signpost | `doctrine memory record`, `doctrine memory verify`, `.doctrine/memory/` | Short: record, verify, find/retrieve, trust holdback. Points to skills for detail. |
| 7 | `mem.signpost.doctrine.backlog` | signpost | `doctrine backlog`, `.doctrine/backlog/` | Work intake model, membership test, kinds, promotion to slice. Cites `mem.concept.backlog.work-intake-membership`. |
| 8 | `mem.signpost.doctrine.adrs` | signpost | `doctrine adr`, `.doctrine/adr/` | When to author, lifecycle states (`proposed→accepted→superseded`) |
| 9 | `mem.signpost.doctrine.specs` | signpost | `doctrine spec`, `.doctrine/spec/` | PRD → SPEC → REQ hierarchy. Entity relationships, not workflow. Skills have workflow; CLI has commands. |
| 10 | `mem.signpost.doctrine.requirements` | signpost | `doctrine coverage`, `doctrine reconcile`, `doctrine spec req` | Coverage store (observed tier), VT verifier (SL-057), reconcile writer. Honest about dogfooding state. |
| 11 | `mem.signpost.doctrine.audit` | signpost | `doctrine review`, `.doctrine/state/` | Audit phase: evidence gathering, reconciliation, close gate. Phase-level, not RV-ledger-level. |
| 12 | `mem.signpost.doctrine.revisions` | signpost | `doctrine revision`, `.doctrine/revision/` | REV kind (ADR-013), change-axis, governance dependency routing |
| 13 | `mem.signpost.doctrine.policies-standards` | signpost | `doctrine policy`, `doctrine standard` | Governance standing rules. The `boot --check` warning will point here. |
| 14 | `mem.signpost.doctrine.review` | signpost | `doctrine review` | **Deferred** until SL-068 lands. Will cover RV kind, turn-based ledger, baton. |

### 2.3 Deferred

- **Review (RV kind)** — SL-068 is in flight; the review surface is changing.
  Author a review signpost in a follow-up slice after SL-068 stabilises.
- **Worktree/dispatch** — Tier 4, genuinely advanced/doctrine-internal.
- **Thread visibility** — corner case, Tier 4.

## 3. Boot snapshot memory section trim (IMP-007)

### 3.1 Current behaviour

`src/boot.rs` renders the Memory section as a full table of every **active**
memory in the store — shipped and project-local alike
(`memory::list_rows` with `type_f=None`, status-filtered to `active`). In
this repo that is ~137 rows. Shipped memories are a modest fraction;
project-local implementation patterns and dispatch internals dominate the
output. The boot digest is ~140 lines of memory rows — material bloat in the
compactness-constrained snapshot.

### 3.2 Target behaviour

Filter to **signpost-type memories only**. Signposts are navigational pointers
— exactly the orientation content the boot snapshot exists to push. After
SL-069 this reduces ~137 rows → ~16 rows (the 5 existing signposts + the 11
new shipped signposts). Compact, stable. Concepts, patterns, facts, and
project-local implementation memories remain discoverable via `doctrine
memory find`/`retrieve`.

### 3.3 Implementation

`src/boot.rs` memory-render path. Filter: `memory_type == "signpost"` before
table construction. No config toggle — the filter is the policy. No ordering
change; the existing `boot_sequence` order is preserved. Concepts and facts
remain discoverable via `doctrine memory find`/`retrieve` and through
cross-references in the shipped signposts.

### 3.4 Verification

- **VT** — new boot snapshot golden test: snapshot markdown Memory section
  contains only signpost-type rows.
- **VT** — assertion: rendered Memory section contains zero non-signpost rows.
- **VT** — non-signpost shipped memories present in `doctrine memory list`
  (they're still shipped, just not rendered in boot).

## 4. boot --check governance warning (IMP-015)

### 4.1 Current behaviour

`boot --check` validates snapshot presence and staleness. Empty governance
sections (Active Policies, Active Standards) are silent — a new project with
no policies gets no signal that governance is unpopulated.

### 4.2 Target behaviour

`boot --check` emits a **warning** when a governance section heading exists
but has no content rows. Also: the boot snapshot itself gains a one-liner
nudge in each empty section pointing users at
`mem.signpost.doctrine.policies-standards`. This is a gentle prompt to bed in
project governance before too much work accumulates — introducing the concept
and the tooling in one signal.

### 4.3 Implementation

`src/boot.rs` — in the check path, after parsing the rendered snapshot, inspect
the Policies and Standards sections. If a section header is present but row
count is zero, emit a warning row. In the snapshot render path, append a
one-line comment to empty sections: `<!-- No active policies yet. See
mem.signpost.doctrine.policies-standards -->`. The comment is invisible to
the agent reading the snapshot but surfaces as a hint if the raw markdown is
inspected; the `boot --check` warning is the active signal.

### 4.4 Verification

- **VT** — `boot --check` emits warning row for empty Policies section in a
  fresh install (no policies authored).
- **VT** — `boot --check` emits warning row for empty Standards section.
- **VT** — populated sections produce no warning (existing tests green unchanged
  for the boot snapshot golden).

## 5. Existing corpus refresh (PHASE-04)

### 5.1 Problem

The existing 14 shipped memories were authored 2026-06-05/06. Since then ~30
slices have shipped, adding 16+ CLI verbs (`review`, `rec`, `revision`,
`reconcile`, `coverage`, `inspect`, `survey`, `next`, `blockers`, `explain`,
`policy`, `standard`, `knowledge`, `worktree`, `dispatch`, `validate`, `reseat`,
`link`, `unlink`, `needs`, `after`, `supersede`), new entity directories
(`.doctrine/review/`, `rec/`, `revision/`, `policy/`, `standard/`, `knowledge/`),
and installed files (`using-doctrine.md`, `glossary.md`, `doctrine.toml.example`,
templates, rules/AGENTS.md).

### 5.2 Approach

**Holistic review, not mechanical fix.** After PHASE-01 authors the 13 new
memories, PHASE-04 reads every shipped memory (existing + new) against the
current codebase and design.md to produce a consistency report. Findings
include:

- Stale CLI verb lists (e.g. `mem.signpost.doctrine.cli-command-map` missing
  the ~16 post-018 verbs)
- Missing directories in `mem.signpost.doctrine.file-map`
- Concepts that have evolved (e.g. the lifecycle now includes `review` and
  `reconcile` phases from SL-040/ADR-009)
- Cross-reference opportunities between existing and new memories
- Duplication or near-duplication between existing memories and
  `using-doctrine.md`

The phase updates affected existing memories in place (`memory/` tree) and
commits them. No new memories are created in this phase.

### 5.3 Handover

The PHASE-04 execution is designed for handover to a fresh agent: the design
provides the consistency criteria, the new corpus (PHASE-01 output) provides
the cross-reference target, and the agent performs the read-and-revise pass.

**Handover checklist — prioritize these five.** The agent must update these
memories, which carry the highest-severity staleness:

1. `mem.signpost.doctrine.cli-command-map` — missing ~16 post-018 verbs;
   lists `skills` which no longer exists (`claude`).
2. `mem.signpost.doctrine.file-map` — missing `.doctrine/review/`, `rec/`,
   `revision/`, `policy/`, `standard/`, `knowledge/`.
3. `mem.signpost.doctrine.skill-map` — paths say `plugins/doctrine/skills/`;
   actual installed path is `.doctrine/skills/`.
4. `mem.signpost.doctrine.lifecycle-start` — missing `reconcile` phase
   (ADR-009 closure seam: `audit → reconcile → done`).
5. `mem.pattern.doctrine.core-loop` — same lifecycle gap.

**Do not rewrite.** storage-model, entity-engine, memory-model, routing-gate,
tdd-loop, cli-source-of-truth, storage-tiers, and overview are substantively
correct. Cross-reference additions and minor wording fixes only.

## 6. Open question: self-updating shipped memories (OQ-1)

**How do we ensure shipped memories stay current as doctrine evolves?** The
corpus was authored in SL-018 (2026-06-06), drifted silently for ~30 slices, and
SL-069 is the first catch-up pass. Without a mechanism, SL-069's output will
similarly rot.

Candidates (not decided here — surfaced for design discussion):

- **Audit-at-close.** Each slice's close process includes a check: "do any
  shipped memories need updating for this slice's changes?" Feels manual and
  easy to skip.
- **Boot --check extension.** `boot --check` already detects stale snapshots. It
  could also compare shipped memory commit dates against the current HEAD date
  and warn on memories older than N days / N commits. But "age" ≠ "stale" —
  some memories are genuinely evergreen.
- **CLI surface comparison.** A `boot --check` variant that diffs the shipped
  CLI command map against the live `doctrine --help` output and flags verbs not
  mentioned. Mechanical, automatable, but only catches one class of staleness
  (CLI verbs).
- **Slice-scope discipline.** When a slice changes a subsystem, the governing
  design or plan carries a "shipped memory impact" section. Lightweight but
  depends on author discipline.
- **Do nothing different.** Accept periodic catch-up passes (SL-069 pattern)
  as the cost of doing business. The simplest approach; risks long drift
  windows.

This is recorded as an open question for the design review. The slice does not
commit to a mechanism — it flags the problem.

## 7. Implementation phases

| Phase | What | Depends on |
|-------|------|------------|
| PHASE-01 | Author 13 new shipped memories under `memory/`. Each: `memory.toml` (ADR-002 signature, master-lint pre-commit gate per §2.4), `memory.md`, `mem.<key>` symlink alias. | — |
| PHASE-02 | Boot snapshot memory trim: filter to signpost-only in `src/boot.rs`. Update goldens. | — |
| PHASE-03 | `boot --check` governance warning + empty-section nudge comment. | — |
| PHASE-04 | Holistic corpus refresh: read all 27 memories against current codebase, update stale existing entries, cross-reference new ones. Handover-friendly (see §5.3 checklist). | PHASE-01 |
| PHASE-05 | Integration: `cargo build` re-embeds, `doctrine memory sync` materialises, `memory find`/`retrieve` surface test, full `just gate`. | PHASE-01, 02, 03, 04 |

PHASE-02 and PHASE-03 are file-disjoint from PHASE-01 (different source files)
and from PHASE-04 (PHASE-04 touches `memory/`, not `src/boot.rs`). PHASE-02 and
PHASE-03 share `src/boot.rs` and should be one phase or tightly sequenced.

### File-disjointness for parallelism

- PHASE-01: `memory/` tree only
- PHASE-02 + PHASE-03: `src/boot.rs`, boot golden tests
- PHASE-04: `memory/` tree (same as PHASE-01, so sequential after)
- PHASE-05: integration, no new files

PHASE-01 and PHASE-02/03 are file-disjoint and could run in parallel.
PHASE-04 depends on PHASE-01 output. PHASE-05 is serial final gate.

## 8. Verification alignment

| What | How |
|------|-----|
| New memories pass master-lint | `corpus::lint_master` on each new `memory.toml` (PHASE-01 pre-commit gate per §2.4) — existing lint tests green unchanged; new golden for full 27-memory corpus |
| New memories surface in retrieval | One integration test per new memory (13 tests): `doctrine memory find --path-scope <scope>` after sync returns the memory by key. Test fixture primes `shipped/` via `memory sync`; asserts exact key match. |
| Boot snapshot renders signpost-only | Golden test: snapshot markdown contains only signpost-type rows in Memory section |
| Boot snapshot contains governance nudge | Golden test: empty Policies section carries the nudge comment |
| `boot --check` warns on empty governance | New test: `boot --check` on fresh-install state emits warning rows |
| Existing memory tests unchanged | SL-005/007/008 suites green unchanged (behaviour-preservation gate). **Risk:** updating stale CLI verbs in `cli-command-map` or adding cross-references changes shipped memory body text. If retrieval tests assert on body content, those tests will legitimately fail — the body changed, not the behaviour. The PHASE-04 agent must run the gate before committing, distinguish legitimate body-change failures from breakage, and update test oracles only when the body change is intentional and correct. |
| `doctrine memory sync` materialises full corpus | Existing sync tests green unchanged (behaviour-preservation); new test: sync from clean state, assert `shipped/` contains exactly 27 INV-signatured dirs. New integration tests should also assert the ADR-002 shipped signature on each new memory (`repo=""`, `anchor_kind="none"`), not just key match. |
| Full gate clean | `just gate` — zero clippy warnings, all tests pass |

## 9. Design decisions

| D# | Decision | Rationale |
|----|----------|-----------|
| D1 | Author fresh, don't promote verbatim | Non-shipped memories carry build-repo framing and stale caveats |
| D2 | Signpost-only boot snapshot Memory section | Orientation purpose; compactness; concepts/patterns reachable via cross-refs |
| D3 | Separate phase for existing corpus refresh (PHASE-04) | New authoring first so refreshed memories cross-reference new ones |
| D4 | Defer review (RV kind) memory until SL-068 lands | Surface in flux; avoid immediate staleness |
| D5 | Boot nudge as markdown comment + `boot --check` warning | Gentle prompt, not blocking error; introduces the capability |
| D6 | 13 new memories (not 8, not 17) | Tiers 1–3 covered; Tier 4 (worktree/dispatch) genuinely advanced; review deferred |
| D7 | `signpost` type for most new memories | Navigational pointers match the corpus model; `concept` only for reading-entities and boot-snapshot (durable mental models) |

## 10. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| New memories contain stale CLI shapes | Medium | Low — but undermines the corpus's authority | PHASE-04 holistic review catches drift; `doctrine --help` is the fallback authority |
| Boot snapshot bloat from nudge comments | Low | Low | Comments are invisible to agents, one-line each |
| PHASE-04 review scope creep | Medium | Medium — could expand into rewriting all 14 existing memories | Design constrains to staleness fixes + cross-refs only; handover instructions bound scope |
| Review memory deferred indefinitely | Low | Medium | SL-068 is active (ready status); capture follow-up in slice close if not yet landed |
| Self-updating problem unsolved (OQ-1) | High | High — another SL-069 needed in 30 slices | Explicit OQ in design ensures it's not forgotten; design review may pick a mechanism |

## 11. References

- ADR-002: Global orientation memory class
- ADR-005: Shipped knowledge tiering by access pattern
- research.md: Full shipped corpus catalogue, gap analysis, non-shipped findings
- IMP-007: Trim boot snapshot memory section
- IMP-015: boot --check flags empty governance sections
- SL-018: Original shipped corpus implementation
- SL-057: Formal VT verification (done, stable for requirements signpost)
- SL-068: Dispatch candidates for safe audit interaction (in flight, review deferred)
