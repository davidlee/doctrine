# RFC-011 — Friction taxonomy & remediation ledger

> Contributed by an autonomous improvement pass (branch `fable-loop`, 2026-07-03).
> This is a **synthesis of `case-notes.md`**, not a replacement — `case-notes.md`
> stays the raw instrument. Each friction below is classified by root cause and
> paired with its disposition (fixed here / OBE / by-design / deferred). Commit
> refs are on `fable-loop`; the User cherry-picks what to keep.

## 1. Friction classes (the taxonomy)

The case-notes collapse to six root-cause classes. Token cost is dominated by the
first three: they force a *wrong action* or a *defensive re-derivation*, not just
a re-read.

| Class | What it is | Token failure mode | Case-note instances |
|---|---|---|---|
| **C1 CLI-shape guess-miss** | A command's real arg shape (flag vs positional, required flags) differs from how a skill/doc models it | Agent copies the shape → hard error → recovery round-trips | `slice phase --status` (positional-looking), `memory retrieve` (no positional key), `worktree land --fork` (branch vs path), `backlog edit` (docs said "prompts"), `arm-spawn --slice` (diagnostic-only) |
| **C2 Governance-mechanics gap** | A gate/algorithm's *granularity or surface* is undocumented, so an agent reasons at the wrong level | Wrong claim asserted, caught late (external review) → rework | ADR-001 edge granularity (top-level collapse); `slice conformance` reads the selector registry, not §6 prose |
| **C3 Brief/skill surface mismatch** | One skill emits an instruction naming a surface a *downstream* skill won't write | Downstream stalls, asks the human, or edits the wrong tier | reconcile brief named a `plan.toml` edit (immutable-append, off-surface); prose-vs-registry duality |
| **C4 Empty-body reverse-engineering** | An entity's actionable content lives only in the title / external memory, not its body | Multi-round scavenger hunt to reconstruct the task (~7k tokens for a 2-line change) | IMP-103 empty IMP body templates |
| **C5 State-vs-summary drift** | A compaction summary / snapshot lags on-disk reality | Defensive re-probing of true state on every resume | sl181 post-compact resume (~6 probes); stale boot Memory index |
| **C6 Env/provisioning** | A worktree/jail is missing a gitignored build input or has a baked path | Build/test fails opaquely before any real work | hand-created worktree lacked `web/map/dist/`; `cargo test` marker-worktree false-fails; `env!(CARGO_MANIFEST_DIR)` baked path |

## 2. Remediation ledger (this pass)

Fixed at root, each a clean commit, verified against code before editing:

| Friction | Class | Fix | Commit |
|---|---|---|---|
| ADR-001 edge granularity undocumented (drove a false tangle-safety proof) | C2 | Added a "Gate edge model" subsection: edges are top-level→first-segment, BTreeSet-deduped; sub-classification refines the direction check only, never the tangle ratchet. Verified vs `tests/architecture_layering.rs`. | `ceff6e90` |
| reconcile brief named an off-surface `plan.toml` edit; conformance prose-vs-registry duality | C3 | `/audit` brief-writing step forbids `plan.toml` items (immutable-append) and requires conformance findings to name the `slice selector` registry verb; `/reconcile` catches a bad item and hands back. | `a7e9ec0e` |
| `*/new` slug symlink stranded by path-scoped `git add <dir>/NNN/` | C1/C6 | `/close` commit guidance: stage `git add <kind>/NNN*`; check for a dangling `NNN-slug`. | `fe3eed9a` |
| `worktree land --fork` bare `no-such-fork` when a *path* was passed | C1 | Pure `no_such_fork_message` enriches that one refusal with a path-vs-branch hint; VT-golden token preserved (e2e `.contains`). | `e0e7b7b1` |
| `backlog edit` documented as "(prompts)" — actually requires `--status` | C1 | Corrected the verb table + two prose claims; `handover` `slice phase … in_progress` positional→`--status` flag. Audit verified all other verb shapes correct. | `305c8638` |
| `CoverageStatus` rendered via `{:?}` Debug (`InProgress`), asymmetric with kebab input | C1-adjacent | Single-source `as_kebab` formatter (= `parse_status` vocab) at both render sites; round-trip test locks the pair. | `07f9a4a2` |
| cordage `explain()` foreign node returned per-overlay singletons, rustdoc promised empty | correctness | In-range guard in `explain` (`node.0 >= node_count` → empty cone). | `1dacc7a8` |
| `next` value cell showed ABSENT for value-bearing kinds scored at default 1.0 | correctness | `value_cell` renders the effective default (`1.0*`) for value-bearing kinds, ABSENT for valueless; `DEFAULT_VALUE` single-sourced. The e2e golden had itself encoded the bug. | `e30e482e` |
| top-level `needs`/`after` stored AND echoed the raw CLI arg (`needs SL-1` → `needs=["SL-2"]`), diverging from the backlog path | C1/correctness | `resolve_dep_seq_src` returns the canonical ids; all three callers use them for the leaf write and the echo, so storage and echo agree. | `6960789d` |
| `estimate set` collapsed only-lower / only-upper / neither into one generic error | C1 | Split into specific arms naming the missing bound (`LOWER without UPPER`, …). | `44907fbc` |
| golden_net proved level determinism but not level *values* (a golden could encode a wrong recurrence — obs #3) | test hardening | Added an independent longest-path value oracle asserting `order_key` levels under every permutation. | `4a3f1412` |

## 3. Not fixed — and why

- **By-design (won't-fix):** worker confinement blocking authored `.doctrine/`
  writes (ADR-008); `cargo test` red under a worker marker (the marker IS the
  mechanism; the funnel's marker-cleared diff is the real check); `slice phase
  --status` / clippy `unwrap_used` denials / `References`-needs-`Role` (standard
  CLI/lint/relation shapes); the vtgate comment-keyword match (an accepted POL-002
  weakness — threat model is omission, not an adversary).
- **OBE (already fixed on `edge`):** `memory retrieve` phrasing (boot-footer
  reworded + `doctrine_onboard` MCP path); ISS-058 `append_edge` contiguity;
  and a whole block of recent IMP items (IMP-112/177/185/214, ISS-035). *Lesson:
  verify-before-trust — roughly half of the older case-note frictions were stale.*
- **Deferred (larger than one clean autonomous increment):** `arm-spawn --slice`
  coord-resolution guard (hot dispatch/SL-190 code); IMP-183 estimate/value
  render on non-slice show surfaces (signature churn across the backlog render +
  JSON parity + goldens — borderline slice-worthy); ISS-205 cordage `env!` baked
  path (no clean local red — passes when compiled in place).
- **Deferred — needs a design decision, not autonomous invention:** ISS-059
  `review prime` aborts (`IsADirectory`) when a selector resolves to a directory
  or symlink→dir (a memory-master dir). Root: `resolve_selectors_to_fileset`
  passes a literal selector through unexpanded, so a dir reaches `contentset::
  compute`'s `fs::read`. No existing dir-content-hash pattern to reuse. Two
  candidate fixes, each defining new drift semantics: (a) **[recommended]** expand
  a dir/symlink→dir selector to its tracked files in `resolve_selectors_to_fileset`
  (canonicalize → `git ls-files` the real dir → per-file entries; drift stays
  per-file, consistent with globs); (b) hash the directory's contents in `compute`
  (walk+combine — defines per-dir walk order / symlink depth / removal semantics).
  Left for a slice.

## 4. Cross-cutting observations

1. **C1 dominates and is the cheapest to prevent.** A skill that cites a wrong
   command shape taxes *every* agent that copies it. A one-time audit of skill
   command citations against `--help` ground truth (done here — 22 skills, 4
   real traps) is high-leverage and repeatable as a lint.
2. **The `--help` text is the authority; skills drift from it.** Consider a CI
   check that extracts `doctrine …` citations from skill bodies and diffs them
   against the CLI's actual arg model.
3. **Goldens can encode the bug.** The IMP-211 `next` golden pinned the wrong
   output (a value-bearing row shown as `·` while scored 1.0). A golden asserts
   *current* behaviour, not *correct* behaviour — pair value-surface goldens with
   an independent oracle (cf. IMP-019's gap).
4. **Empty authored bodies (C4) are a silent tax.** The storage rule keeps bodies
   free-form, but a non-obvious item whose problem statement lives only in the
   title or an external memory forces reconstruction. Capturing the actionable
   statement in the body at creation time is the cheap fix (author discipline,
   not tooling).

---

# 2026-07-28/29 batch — 68 observations across SL-233 dispatch & SL-237 design

> Analysis written 2026-07-30. Covers 68 friction observations recorded
> 2026-07-28 through 2026-07-29, spanning a single dispatch slice (SL-233,
> 14 phases) plus one design gate (SL-237). Every observation is classified
> against the same C1-C6 taxonomy; four new classes are proposed where the
> existing bins fail to capture a distinct root cause.

## 1. Class frequency (existing + new)

| Class | Count | % | Dominant mechanism |
|---|---|---|---|
| **C7 — Cross-tree state leak** | 15 | 22% | Agent reads/writes the wrong tree with no signal |
| **C3 — Brief/skill surface mismatch** | 14 | 21% | Worker brief under-specifies scope or gets constraints wrong |
| **C1 — CLI-shape guess-miss** | 10 | 15% | Error messages invite wrong next attempts; read surfaces missing |
| **C6 — Env/provisioning** | 10 | 15% | Worker fork environment breaks self-checks; jail constraints |
| **C8 — Decision-location asymmetry** | 5 | 7% | Guard fires at the wrong lifecycle point |
| **C9 — Read-surface poverty** | 5 | 7% | No read verb exists — not a shape mismatch, a total absence |
| **C2 — Governance-mechanics gap** | 3 | 4% | Gate operates at wrong granularity |
| **C10 — Oracle under-specification** | 3 | 4% | Verification criterion is true but doesn't catch the defect |
| **C5 — State-vs-summary drift** | 2 | 3% | Snapshot/staleness without tree confusion |
| **C4 — Empty-body reverse-engineering** | 1 | 1% | Missing procedure document |

## 2. New friction classes

### C7 — Cross-tree state leak

The agent operates across multiple worktrees (primary/edge, coord, fork) and the
harness or tooling silently resolves a relative path, a cwd, or a state file
against the *wrong* tree. The stale tree answers confidently; nothing signals
which tree was read or written.

**Why it's not C5 (state drift):** the state IS accurate — in the wrong tree.
The mechanism is resolution ambiguity, not staleness.

**Why it's not C6 (env):** the environment is correctly provisioned; the problem
is that nothing distinguishes the two correct environments.

**Instances (15):**
- Bash cwd silently resets to primary tree between calls (#32, #19 — two
  independent hits, one nearly mutating the primary tree on edge)
- Worker reads primary tree's stale plan.toml instead of fork's (#4)
- handover.md stale copy in primary tree read instead of coord copy (#40)
- ISS-274: funnel verbs leave staged deletions; stale index committed, silently
  reverting the verb that just ran (#6, #64)
- review verbs refuse in the coordination worktree — blanket
  `is_linked_worktree` guard (#62)
- reservation `reach=local` collides review ids across coord and primary (#61)
- scratchpad dir vanished between Bash calls in a worker (#38)
- record-boundary leaves staged deletion (#64)
- funnel prescribes spawn for coordinator-only phases (#65)
- `.doctrine/state/` flat namespace: 150+ per-session scratch files at top
  level make any survey unaffordable (#63)
- .agents/ skill mirror lag read as failed install (#24)
- plugins/ skill edits cannot reach .agents/ (#48, #49)

**Dominant failure mode:** a relative path typed by habit (`/workspace/doctrine/...`)
or a cwd-dependent path resolves to the primary tree when the agent intended the
coord or fork tree. The result is always plausible — the stale file exists and is
well-formed — so the agent proceeds with wrong data until something else breaks.

**Cheapest structural fix:** every dispatch brief that cites a file path should
make it worktree-relative and say so explicitly; the handover packet should stamp
its source tree path and branch. Longer-term: the harness cwd-reset note should
attach to the call that *gets* the new cwd, not the one that lost it.

### C8 — Decision-location asymmetry

A validation gate has the information it needs to refuse early but does not
check until the end of a long pipeline. The agent pays the full cost of the
pipeline before discovering the refusal.

**Why it's not C2 (governance gap):** the governance IS correct — it correctly
refuses. The gap is *where* the check fires in the lifecycle.

**Instances (5):**
- `arm-spawn --slice N --phase PHASE-NN` does not refuse when that phase is
  already at `verified`; `worker_commit` does — after the full worker run (#2, #3)
- `arm-spawn --slice` without `--phase` yields an unbound fork that only
  surfaces after the whole worker run (#53)
- Worker brief prescribed a seam shape that violated VA-1; the grep signal
  existed but wasn't named, so discovery cost a build cycle (#37)
- `/phase-plan` caught EX-11/EX-12's test-location conflict but missed the
  same fact's second consequence; the check was point-fix not sweep (#52)

**Dominant failure mode:** `arm-spawn` knows the slice and phase — that's enough
information to look up the funnel position and refuse at arming time. The
refusal at `worker_commit` is correct but comes after 14+ minutes of work.

**Cheapest structural fix:** `dispatch arm-spawn` should refuse (or warn) when
the phase's funnel position is past `imported`. The information is available at
arming time.

### C9 — Read-surface poverty

A needed read operation has no CLI verb. Not a shape mismatch (C1) — the verb
doesn't exist at all. The agent falls back to grepping raw authored files.

**Why it's not C1:** C1 is about existing verbs with wrong shapes. C9 is about
absent read surfaces — the surface was never built.

**Instances (5):**
- `slice phase <id> <PHASE>` looks like a read but is a status *setter*
  (requires `--status`); no `slice phase show` exists (#54)
- `slice plan` is a writer; no read-back verb for a phase's criteria exists (#27)
- No CLI verb prints a review ledger's findings — must read `review-NNN.toml` raw (#28, #41)
- `spec req` lacks an entity show surface (#35)
- `slice show` omits outbound `fulfils` edges that `inspect` renders (#11)
- `backlog list/show` prints a dangling-override block on every invocation (#30)

**Dominant failure mode:** the agent greps raw TOML against the boot rule that
says "read entities via show, not raw files." The `show` verb exists but is
incomplete, or the needed verb was never written.

### C10 — Oracle under-specification

A verification criterion or worker brief names a check that is true but
doesn't catch the defect it's meant to guard against. The oracle passes while
the defect remains.

**Why it's not C3:** C3 is about naming the wrong *surface* (a file that won't
be written). C10 is about the criterion itself being too weak — it describes a
necessary condition that is insufficient.

**Instances (3):**
- Anti-theatre criteria paid off: two mandated tests were green and
  meaningless until mutation testing exposed them (#50)
- Published evidence table outran the committed instrument — the probe
  implemented 3 of 5 rules, missing the two rows that carried the argument's
  load-bearing number (#22)
- `reseat RV-NNN` fails at parse — reviews store no status by design, so
  the verb's contract is unreachable (#42)

**Dominant failure mode:** a criterion passes but the thing it's checking is
still broken. The criterion measures the wrong property.

## 3. Existing classes — what's new in this batch

### C1 — CLI-shape guess-miss (10 instances)

This batch's C1 instances are dominated by **error messages that invite wrong
next attempts** rather than wrong documentation:

- `search --kind` error says "Valid groups: backlog, ..." — the natural next
  attempt is `--group backlog`, which doesn't exist (#9)
- `review --as` takes a closed vocabulary (raiser/responder) while `review new
  --responder` takes a free-text label — same concept, different surface (#29)
- `memory retrieve` takes only `--tag`, so free-text retrieval needs
  `memory search` (#14)
- `slice research` advisory mints a baseline as a side effect — read-shaped
  verb with a write side effect (#55)
- Multi-file grep silently omitted matches from the first file argument (#57)
- Backlog documented shell is `nu` but harness Bash tool is POSIX (#20)

### C3 — Brief/skill surface mismatch (14 instances)

Dominated by **dispatch worker briefs** that get scope or constraints wrong:

- Under-declared editable file set vs slice selector (#7)
- Undercounted migration blast radius (4 fixtures named, ~8 unlisted) (#16)
- Predicted wrong red kind: said wrong-acceptance, was wrong-refusal-reason (#17)
- VA-NC mandated a red kind unreachable for 2 of 5 refusals (#8)
- Seam shape violated VA-1 (#37)
- Fixture contradicted VA-3 (#13)
- Brief asked for multiple commits but worker_commit lands exactly one (#43, #46)
- Brief mandated unreachable assertions — private-field assertions while VA-7(3)
  forbade the read_to_string they depended on (#44)
- RV-321 F-4 mandated `crate::kinds::SLICE_DIR` in a test unreachable due to
  submodule dependency (#45)
- EN-2 specified a marker grammar without noting an incumbent implementation
  already existed at head (#31)
- External reviewer (codex, workspace-write) rewrote slice notes.md Harvest
  section during design review (#33)
- External reviewer couldn't find /route at conventional path (#47)

### C6 — Env/provisioning (10 instances)

Dominated by the **worker fork self-check gap** — the worker cannot trust its
own gate run:

- ~10 integration targets unconditionally red in every marked worker fork —
  tests spawn the binary without `.current_dir()`, inherit the worker marker (#36, #51)
- Binary-only crate forces ~120-line fixture duplication across test binaries (#12)
- pi-scout/pi-research hang indefinitely on tool-using prompts (#59, #60)
- Read-only git index blocks path-limited commit in a jail (#34)
- grep resolves to a gitignore-respecting backend; `.agents/` skipped (#23)
- .agents/ skill mirror lag read as failed install (#24)
- plugins/ skill edits cannot reach .agents/ — that tree is npx-fetched (#48, #49)

## 4. Cross-cutting observations (this batch)

1. **C7 (cross-tree state leak) is the new C1.** In the fable-loop batch C1
   dominated (~40% of instances). In the dispatch-heavy 2026-07-28/29 batch
   C7 dominates (22%) — the multi-worktree model is now the primary friction
   surface. This is a natural consequence of moving from solo development to
   dispatch orchestration: every phase involves at least three trees (primary,
   coord, fork) and the harness does not distinguish them.

2. **Worker brief accuracy is the dispatch funnel's sharpest edge.** C3 (14
   instances, 21%) is almost entirely dispatch-specific — the orchestrator's
   distilled brief gets scope, constraints, or file maps wrong, and the worker
   discovers this at implementation time. Every inaccurate brief costs a full
   worker run. The cheap lever is better distill-time validation (grep the
   blast radius BEFORE writing the brief, not after).

3. **The read-surface gap is systematic.** Across C9 (5 instances) and
   C1-read-surface-adjacent (6 more), there are at least 11 distinct
   observations where an agent needed to read something and either couldn't
   or had to resort to raw file inspection. "Read entities via show, not raw
   files" is a boot guardrail that many surfaces don't yet satisfy. The
   `review show` and `slice phase show` gaps are the highest-impact.

4. **Dispatch's `arm-spawn` should be the decision point.** C8's three
   arm-spawn instances (already-verified, no-phase, and the brief-shape
   build cycle) all share the same fix: move validation upstream. The
   information exists at arming time; checking it there saves a full worker
   run per instance.

5. **Worker fork self-check is broken for every worker, every phase.** C6's
   ~10 always-red integration tests (instances #36, #51) are a per-phase tax
   paid by every dispatch worker. The workers cannot trust `check gate` and
   must hand-classify every failure. Fixing the 10 test fixtures to use
   `.current_dir()` removes this permanently for all future workers. This is
   the highest-leverage single fix in the batch.

6. **The observation ledger itself has an indexing gap.** `grep -rl 'kind =
   "friction"' .doctrine/observations/records/` finds 69 files; `doctrine
   observation list --kind friction --limit 100` returns 68. One file is
   unindexed. Additionally, several observation directories contain two
   distinct friction records — the 2-char hex directory scheme is not 1:1.
   Both are minor but worth noting.

## 5. Remediation ledger (this batch — proposed, not applied)

| Friction | Class | Suggested fix |
|---|---|---|
| Worker fork ~10 integration tests always red | C6 | Add `.current_dir(tmp)` + `.env_remove("DOCTRINE_WORKER")` to the 10 affected test binaries |
| `arm-spawn` doesn't refuse on already-verified phase | C8 | `dispatch arm-spawn` should look up funnel position and refuse past `imported` |
| Worker brief under-declares blast radius | C3 | Distill-time grep of every criterion's file footprint before writing the brief |
| Bash cwd silently resets to primary tree | C7 | Harness: attach cwd-reset note to the call that gets the new cwd, not the one that lost it |
| Primary tree plan.toml silently answers fork citation | C7 | Dispatch briefs: cite paths as worktree-relative; handover: stamp source tree path+branch |
| `search --kind` error invites `--group` flag | C1 | Error text: "--kind accepts a prefix OR a group: ..." instead of listing "Valid groups" |
| No `slice phase show` / `review show` read verb | C9 | Add `slice phase show <id> <PHASE>` and `review show <id>` as read-only verbs |
| ISS-274 stale index committed as record-delta | C7 | Funnel verbs should clear their own index entry after writing; add `verify` to the ISS-274 handover list |
| Column-aligned plan.toml defeats `id = "PHASE-` grep | C1 | Write `id = "PHASE-NN"` without column alignment, or document the `id\s*=` regex |
| Orphaned `.doctrine/*.md` reference copies | C5 | Delete the 8 tracked orphans; only governance.md and project-orientation.md are live |
| `review contest` rationale is ephemeral | C1 | Contest should write its rationale to the finding's `contest` field, not just baton `--note` |
| Responder cannot amend stale response | C2 | Add an `amend` path for the responder when the design has moved out-of-band |
| `slice show` omits `fulfils` edges | C1/C9 | `slice show` should render all outbound relations, matching `inspect` |
| `.doctrine/state/` flat namespace blows context | C7 | Session-scoped runtime needs its own subdirectory + reaping (adjacent to IMP-338) |
| Phase sheet goes stale against amended plan.toml | C5 | Materialise-on-write or freshness check: stamp the source plan's mtime in the phase sheet |
| `check gate` verdict unreachable from ~5.5k-line tail | C1 | Terminal one-line verdict: legs run, pass/fail each, warning count |
| `arm-spawn --slice` without `--phase` unbound | C8 | Require `--phase` when `--slice` is given; refuse at parse time |
