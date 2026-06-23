# Design — SL-147: Audit path-conformance delta (RFC-004 v0.1)

## Design problem

At audit, the conformance gap between *what the design declared it would touch*
and *what git shows it touched* is hand-hunted. RFC-004 v0.1 makes it computed:
mint an accreting per-slice **selector list** (path|glob + intent), record each
phase's **source-delta SHA** as an owned contract, and add an auditor verb that
emits the declared-vs-actual set algebra. This is the prove-value prototype for
RFC-004's "killer consumer"; the broader path-intent primitive (non-entity edge
targets, per-phase attribution, prose anchoring) stays deferred.

The load-bearing governance constraint is **POL-002**: the delta must rest on
contracts doctrine owns (recorded oids, the dispatch funnel), never on a host's
`(SL-NNN)` commit convention.

## Current state

- `review prime <domain_map.toml>` (src/review.rs:2564) hand-authors a `Cache`
  of `[[area]]`(name/purpose/paths) + `[[invariant]]`/`[[risk]]` prose +
  `[hashes]`. The prose tier has **zero runtime readers** (RFC-004 OQ-5, settled
  dead); `validate_domain_map` (review.rs:2612) only checks non-emptiness.
- The sole live reader is `run_status` (review.rs:2682) → `cache_staleness`
  (2514) / `stale_paths` (2718), via `tracked_paths` (2464) + `baseline` (2475),
  surfaced as `review status`. It reports which tracked paths drifted vs the
  cached hashes.
- The **dispatch arm already records per-phase code boundaries**:
  `BoundaryRow { phase, code_start_oid, code_end_oid }` → `boundaries.toml` under
  `.doctrine/dispatch/<NNN>/`, committed on `dispatch/<NNN>` (ledger.rs:552,
  `dispatch record-boundary`). `code_start`/`code_end` are resolved to full oids
  (dispatch.rs:535). The **solo `/execute` arm records nothing.**
- There is **no `doctrine audit` CLI** — audit is skill-only.
- `git_text`/`git_bytes` (git.rs:537/521) is the single git invocation seam.

## Forces & constraints

- **POL-002** — no load-bearing on host commit/branch conventions; recorded oids
  only. No `(SL-NNN)` grep anywhere.
- **No parallel implementation** (global standard) — one path-set surface, one
  boundary-row type. Burning `domain_map` is required, not optional, to avoid a
  second parallel path-set.
- **Behaviour-preservation gate** — the review staleness behaviour is shared
  machinery; its computation must stay green while its *source* is swapped.
- **Pure/imperative split** (slices-spec § Architecture) — set algebra is pure;
  git, disk, and registry reads live in the thin shell.
- **Agent ergonomics** — selector authoring must be one command per batch of
  same-intent files, not one call per file.
- **In-loop lifetime** — audit runs before close; the recorded-delta registry is
  runtime state, read in-loop, reaped at close. Post-close auditability is not a
  driver.

## Decisions

### D1 — Intent vocabulary: `scope-relevant` vs `design-target` only

Author declares *intent to touch*; **git supplies the change verb** (A/M/D) on
the actual side. `scope-relevant` = read-relevant (RFC L0); `design-target` =
will-touch (RFC L1). Author-declared verbs (write/delete/create) are the deferred
"verb sub-tags" — excluded. Rationale: the author never predicts what git will
report, so the declared side never duplicates/drifts against git; the audit still
*shows* git's A/M/D for free.

```rust
enum SelectorIntent { ScopeRelevant, DesignTarget }   // serde kebab-case
```

### D2 — Storage: `[[selector]]` table in `slice-NNN.toml` (authored)

The selector list is authored, committed, diffable, and accretes as design edits
it — so it lives in the slice's own authored TOML, on `SliceDoc`. Structured data
in TOML per the storage rule; the `note` is a one-line annotation, not the burned
prose tier.

```toml
[[selector]]
selector = "src/review.rs"              # path | glob — neutral noun (RFC's term)
intent   = "design-target"              # scope-relevant | design-target
note     = "re-point staleness reader"  # optional, one line
```

```rust
struct Selector { selector: String, intent: SelectorIntent, note: Option<String> }
// SliceDoc gains: #[serde(default)] selectors: Vec<Selector>
```

Identity is the `selector` string. `add` is an idempotent **upsert** (re-adding a
selector updates its intent/note), mirroring `doctrine link`.

### D3 — CLI surface: batch-first authoring + the conformance reader

Under the `slice` namespace (slice-scoped; no `audit` namespace exists, and
inventing one would imply a broader audit CLI this slice does not build).

```
# authoring — variadic, one call per same-intent batch (the common path)
doctrine slice selector add SL-147 --intent design-target "src/review.rs" "src/slice.rs" "src/state.rs" [--note "shared"]
doctrine slice selector note SL-147 "src/review.rs" "per-file override"   # rare; 2nd call only when needed
doctrine slice selector list SL-147
doctrine slice selector rm  SL-147 "<glob>" ["<glob>" …]

# funnel/solo recording — writes the arm-neutral source-delta registry
doctrine slice record-delta SL-147 --phase PHASE-02 --start <ref> --end <ref>

# the killer consumer
doctrine slice conformance SL-147

# staleness — re-pointed, surface unchanged
review status
```

Batch `add` carries one `--intent` and an optional shared `--note`; per-file notes
are a separate cheap `note` upsert. The MCP reader (fast-follow, deferred) can take
a structured `[{selector,intent,note}]` payload for full per-file richness in one
shot.

### D4 — Burn `domain_map`; re-point the staleness reader (OQ-A)

`domain_map` is an anchor, not a vehicle. Remove the authored input and the prose
tier; keep the staleness *capability* by sourcing its path-set from the selector
list.

- **Remove:** `CacheNote` (invariant/risk), `CacheArea.purpose`, the prose checks
  in `validate_domain_map`, and `run_prime`'s `domain_map.toml` parse/arg.
- **Re-point:** `run_prime` resolves the slice's selectors (the path-set tier) →
  hash → `write_cache`. `cache_staleness` / `stale_paths` / `run_status` are
  unchanged downstream — only their *source* changes.
- **Slim:** `Cache` → `{ paths, hashes }` (derived baseline, not authored prose).

*Correction (internal review):* this is **not** a pure source-swap. `domain_map`
`area.paths` were literal paths handed straight to `contentset::compute`;
selectors admit globs, so the re-point adds a **glob→fileset resolution** step
before hashing (resolve each selector against the worktree → concrete files →
`contentset::compute`). The staleness *computation* is still preserved; its input
gains a resolution pass.

The review test suite is the behaviour-preservation gate for the staleness
computation. Tests that fed `domain_map.toml` migrate to the selector source —
expected, the input is being burned.

*Open sub-question for the draft (resolved here):* staleness sources the **union
of all selectors** (both intents) — both are "paths this slice cares about"; the
conformance diff keys only on `design-target` (D6).

**F-4 — the re-point is scoped to slice-backed RVs.** `run_prime`/`run_status`
are RV-generic: an RV `--target` may be a phase or a backlog item, not only a
slice. The selector source exists **only** for a slice-backed RV. So: `run_prime`
resolves the RV's `--target` → if it is a slice, source the selector list; if it
is **not** a slice (or the slice has zero selectors), fail clearly with a named
message (no silent empty cache), not a panic. The RV→slice resolution is explicit.
Selectors are *committed* authored slice TOML, read in the parent tree (review
verbs already refuse fork roots, review.rs:1953), so the read is fork-safe by the
existing invariant — no new fork access introduced.

### D5 — Recorded source-delta registry: arm-neutral runtime state (OQ-D)

*Post-lock revision (2026-06-23, per User; not a REV — per-slice design):* the
solo-arm writer changed from an explicit-`record-delta`-required contract to
**deterministic capture on the `slice phase` transitions**. RV-148 reviewed the
earlier framing; the change strictly tightens it (a confirmed CLI hook replaces a
discipline contract), so no re-inquisition is warranted — see the solo bullet and
OQ-conf-1.

One registry both arms write and the auditor reads, resolved to a single shared
location regardless of which worktree the command runs in.

- **Home:** `.doctrine/state/slice/<NNN>/boundaries.toml` under the **primary
  working tree**, resolved by **reusing the existing `worktree::subagent::
  primary_worktree(cwd)`** (subagent.rs:33 — `git worktree list --porcelain`,
  correct across layouts), *not* a new helper and *not* CWD-relative `root::find`.
  Lift/share it as needed; do **not** reinvent (F-5). All commands hit one file
  regardless of cwd worktree. Worktree layout is a doctrine-owned contract
  (ADR-006/012) → POL-002-clean. Runtime, gitignored, in-loop lifetime.
- **Writer = the un-jailed orchestrator / solo agent (F-5).** The only writer is
  the session process (dispatch orchestrator, or the solo agent), never a jailed
  worker — so it can write the primary tree by absolute path. `review.rs:1953`'s
  fork ban does **not** transfer: it guards (i) review verbs invoked *from inside*
  a `WITHHELD` fork and (ii) an interactive per-review CAS baton. We have neither
  — **orchestrator-sole-writer (ADR-006)** means a single atomic write, no baton.
- **Row type:** reuse ledger's `BoundaryRow { phase, code_start_oid,
  code_end_oid }` — no new parallel row type. Extract it to a **leaf module** both
  `ledger` and `state` import (ADR-001: no engine↔engine cycle); this is a struct
  move, not a re-implementation.
- **Write-time validation (F-6).** `record-delta` refuses an under-constrained
  range: assert `code_start_oid` is an **ancestor of** `code_end_oid` and confine
  the recorded range to the source-delta commit class (`--first-parent` / reject
  merge commits). A wrong/wide boundary that would pull trunk into `actual` fails
  loudly at write, not silently at audit.
- **Writers / arms:**
  - **solo** (`/execute`, inline-on-main **or** a solo `/worktree` fork): **capture
    rides the `slice phase` state transitions** — the same CLI write `/execute`
    already issues (SKILL.md: `in_progress` at phase start, `completed` at land). On
    `--status in_progress` the handler stamps `code_start_oid = HEAD` into the phase
    sheet; on `--status completed` it captures `code_end_oid = HEAD`, applies the
    F-6 guard, and **upserts** the phase's boundary row (reopen re-captures; `start
    == end` writes the zero-delta row automatically). HEAD is the phase's true
    code-end here because the flip is issued from the tree where the phase landed
    (primary edge, or the solo fork). Deterministic, on every phase's critical path
    — **not** an off-path "remember to also call X." Supersedes the earlier
    explicit-`record-delta`-required contract.
  - **Arm discrimination (dispatch-compat).** The binding fires **only outside a
    dispatch coordination context**. The dispatch orchestrator flips phase status
    from the **coordination worktree**, whose `HEAD` is the coordination base `B`
    (not the worker's source-delta) — so a HEAD capture there would record the wrong
    range *and* duplicate the dispatch beat. The handler therefore **detects the
    dispatch coordination context** (doctrine-owned signal — a `dispatch/<N>`
    coordination branch / coordination worktree role, *not* a host convention, so
    POL-002-clean) and **skips** capture; the dispatch beat is the sole recorder
    there. The gate keys on *dispatch-coordination*, **not** on
    "linked worktree" — a solo `/worktree` fork is not a dispatch context and still
    captures. (In fact dispatch never calls `slice phase` today, so this is a
    belt-and-braces guard, not a load-bearing assumption — verified, then enforced.)
  - **`slice record-delta`** remains the **manual escape hatch**, not the happy
    path: re-record a corrected range, or bootstrap a slice whose phases predate
    the binding (e.g. SL-147's own early phases). Same writer + F-6 guard.
  - **dispatch**: the orchestrator's existing `dispatch record-boundary` beat
    (funnel step 8, `--code-start B --code-end B+1`) is the sole dispatch recorder.
    It **also writes the arm-neutral registry** (the conformance reader's only
    source), *in addition to* its existing committed `.doctrine/dispatch/<N>/`
    boundary (unchanged — dispatch integrate still depends on it). One row type, one
    reader; the dual home is two consumers, not a parallel impl.
    - **Coverage limit (codex/pi arm).** `record-boundary` is **claude-arm-only**
      today (`skip on codex/pi`, dispatch-agent:67 — the subprocess arm has no fork
      branch to range). So a **codex/pi-dispatched** slice records no rows → its
      conformance degrades to `incomplete` (F-2), never a false-clean. Extending the
      subprocess arm to record is **deferred** (Non-goal); v0.1 conformance covers
      **solo + claude-dispatch**.
  - All covered arms write **without** an off-critical-path act; F-2 (below) stays
    the backstop — a completed phase with no row is still caught, never silently
    clean.
- **Reader:** `slice conformance` reads only this registry.
- **Completeness — fail closed on partial coverage (F-2, BLOCKER fix).** Degrade
  is **not** empty-only. The reader cross-checks recorded rows against the slice's
  **completed phases** (the `state.rs` phase sheets): it expects **exactly one row
  per landed phase** (a phase with no code change records an explicit **zero-delta
  row**, never an omission). On any mismatch — missing phase, extra row, phase
  completed but unrecorded — it emits `incomplete` (names the gap) and refuses the
  confident algebra. Empty registry → `unavailable`. Neither ever fabricates a
  clean diff from a partial actual.

### D6 — Slice-delta + the algebra (pure core)

```
actual : Map<path, StatusSet>     # StatusSet = ordered per-phase events (A|M|D)
       = fold over registry rows of: git diff --name-status <code_start>..<code_end>
```

**Net-status rule (F-3).** A path can be touched across phases (A in phase 1, M in
phase 2; or A then D). The fold does **not** collapse to one ambiguous status —
each path carries its ordered **event set**, and a defined `net()` derives the
displayed verb: contains `D` as the last event ⇒ `removed`; contains `A` and no
trailing `D` ⇒ `added`; otherwise `modified`. Presence in `actual` (the algebra
key) is order-independent; only the display verb uses `net()`. `net()` is pure and
unit-tested across A→M, A→D, M→M, M→D→A orderings.

Per-phase code ranges exclude trunk merges by the **write-time guard** (D5/F-6:
ancestor + non-merge), not by assumption — so interleaved trunk contributes
nothing and **no base ref is needed** (RFC-004 OQ-11). Match git's actual paths
against the **design-target** selectors by glob (a literal path is a degenerate
glob — no tree resolution needed for the diff):

```
conformant  = { p ∈ actual.keys | ∃ design-target selector matching p }   # carries the matched selector
undeclared  = { p ∈ actual.keys | no design-target selector matches p }   # highest signal; carries net() verb
undelivered = { design-target selector | no actual path matches it }
```

**Glob-breadth transparency (F-7).** A broad `design-target` glob (`src/**`) would
silently absorb surprise edits into `conformant`, gutting `undeclared`. Mitigation:
every `conformant` path is reported **with the selector that matched it**, so a
blanket declaration is visible in the output (the audit sees "all 40 paths matched
by `src/**`" and can challenge it). A lint that refuses over-broad design-target
globs is deferred (a follow-up); transparency is the v0.1 guard.

Pure function over `(selectors, actual_map)`. Git + registry + selector reads sit
in the shell. *Correction (internal review):* the glob matcher is **not**
`tracked_paths` (which collects literal paths only). Reuse `glob = "0.3"`
(`glob::Pattern`) + `worktree/allowlist::glob_matches(pat, path)` (allowlist.rs:93)
— lift `glob_matches` to a shared leaf module so both `conformance` and the
staleness resolution (D4) consume it. No new glob dependency.

### D7 — Module layout

- `src/slice.rs` — `Selector`, `SelectorIntent`, `[[selector]]` on `SliceDoc`;
  `selector add|note|list|rm` handlers (upsert/dedup by string).
- `src/review.rs` — burn + re-point per D4; `Cache` slimmed.
- a **leaf module** — `BoundaryRow` extracted here, imported by both `ledger`
  and `state` (ADR-001, no engine↔engine cycle; F-5/R3).
- `src/state.rs` — `boundaries_path(slice)` (resolved via reused
  `worktree::subagent::primary_worktree`), `record_source_delta` writer with the
  ancestor + non-merge guard (F-6), reader, and the completed-phases completeness
  check (F-2, reading the phase sheets). The `slice phase` transition handler is
  the **solo capture hook** (D5): `in_progress` stamps `code_start_oid` into the
  phase sheet; `completed` reads it back, captures `code_end_oid = HEAD`, guards,
  and upserts the boundary row. Bare-repo / not-a-repo → the same clean named error
  as the writer, never a panic that would block a legitimate phase transition.
- `src/dispatch.rs` — the orchestrator records into the arm-neutral registry at
  the landing beat (thin write; sole-writer).
- new pure module (e.g. `src/conformance.rs` or under `slice`) — the algebra
  (D6), unit-tested in isolation.
- `src/commands/*` — `slice conformance`, `slice record-delta`, `slice selector*`
  wiring; the conformance shell (resolve registry + git name-status + selectors →
  call pure algebra → render).
- `src/git.rs` — reuse `git_text` for `diff --name-status`.
- ADR-001 layering: pure algebra is leaf; shell composes it. No cycles.

### D8 — Skill touch-points (light)

- `/slice` — seed coarse `scope-relevant` selectors at cut.
- `/design` — add `design-target` selectors (the load-bearing input).
- `/execute` — **no new step.** Capture rides the `slice phase` transitions
  `/execute` already issues (D5); the boundary is recorded automatically at
  `in_progress`/`completed`. `record-delta` is only the manual fallback.
- `/audit` — run `slice conformance`, read the algebra into the audit.

## Data flow — `slice conformance SL-147`

1. Read `SliceDoc.selectors`; keep `intent == design-target`.
2. Read `.doctrine/state/slice/147/boundaries.toml`. Empty → emit `unavailable`,
   stop.
3. For each row: `git diff --name-status code_start..code_end` → fold into
   `actual: Map<path,Status>` (shell, `git_text`).
4. Pure algebra (D6) → `{conformant, undeclared, undelivered}`.
5. Render: undeclared (with A/M/D) first (highest signal), then undelivered, then
   a conformant count.

## Verification & test plan

- **Pure algebra unit tests** — conformant / undeclared / undelivered / glob vs
  literal / multi-phase event-set; `net()` over A→M, A→D, M→M, M→D→A (F-3).
- **Degrade tests (F-2)** — empty registry → `unavailable`; recorded-rows vs
  completed-phases mismatch (missing / extra / unrecorded-completed) → `incomplete`
  naming the gap; never a clean diff from partial coverage.
- **Write-time guard tests (F-6)** — `record-delta` refuses a non-ancestor range
  and a merge-commit `code_end`.
- **`slice conformance` integration test** — fixture slice with a complete
  recorded registry + synthetic ranges; assert the three cells **and** the matched
  selector per conformant path (F-7).
- **`slice selector` tests** — batch add (variadic), upsert semantics, `note`
  override, `rm`, round-trip through `slice-NNN.toml`.
- **Non-slice RV (F-4)** — `run_prime` against a phase/backlog RV target fails
  with the named message, not a panic or silent-empty cache.
- **Review behaviour-preservation** — the staleness *computation* (the pure
  `diff`) stays green; only its input fixtures migrate from `domain_map.toml` to
  the selector source (the invariant is the computation, not the input format).
- **POL-002 conformance (VH)** — reviewer challenge: no `(SL-NNN)` grep; delta
  rests only on recorded oids.
- **Prove-value (VA/VH) — concrete, achievable target (F-8).** The earlier
  dogfood was circular (SL-147's own boundaries presuppose the recorder works).
  With the deterministic binding, SL-147's **post-binding** phases (PHASE-04
  onward) auto-record as they complete — run `slice conformance SL-147` and confirm
  real signal. Pre-binding phases (P1..P3) either get a one-time `record-delta`
  bootstrap or read `incomplete` (a live F-2 demonstration). A separate forward
  slice run end-to-end is the fully-clean, zero-bootstrap proof.

## Non-goals (reaffirmed)

Per-PHASE attribution · author-declared verb sub-tags · target sum type /
non-entity edge generalization (OQ-6) · new verify mode VG (OQ-9) · prose
invariants/risks · dispatch disjointness · IMP-012 wiring · durable post-close
registry · MCP reader (fast-follow) · **codex/pi-dispatch boundary recording**
(the subprocess arm's `record-boundary` is claude-arm-only today; codex/pi-
dispatched slices degrade to `incomplete`, never false-clean — extending that arm
is a follow-up). Conformance is necessary-not-sufficient: it says *where to look*,
never *whether it passes*.

## Residual risks

- **R1 — dispatch `integrate` projection.** The one touch to a live dispatch
  path. Mitigate: keep it a thin read→write of an existing struct; gate on the
  existing dispatch tests.
- **R2 — glob `undelivered` semantics.** A forward-looking glob ("will create
  src/foo/*") that matches nothing in `actual` reads as undelivered; that is the
  correct signal (declared, not delivered), but a glob matching nothing because
  the work is *pending* (mid-slice) is noise. Mitigate: conformance is an
  audit-time (slice-complete) read; document the timing assumption.
- **R3 — `BoundaryRow` reuse across modules (RESOLVED).** ADR-001 forbids an
  engine↔engine cycle, so the row is **extracted to a leaf module** both `ledger`
  and `state` import (D5). A struct move, not a re-implementation — confirmed a
  non-violation under that move (external review concurred).
- **R4 — review suite churn.** Burning the input changes test fixtures; the
  computation behaviour is the invariant, not the input format.
- **R5 — cross-worktree registry visibility (RESOLVED → option (a), F-5).**
  `root::find` is CWD-relative, and the dispatch orchestrator `cd`s into the
  coordination worktree, so a coordination-tree registry is invisible to
  `conformance` in the main tree. **Resolution:** reuse the **existing**
  `worktree::subagent::primary_worktree(cwd)` (subagent.rs:33) — not a new helper —
  for both `record-delta` and `conformance`; one shared file under the primary
  tree (D5). Safe because the sole writer is the un-jailed orchestrator/solo agent
  (ADR-006), a single atomic write, no baton — the `review.rs:1953` fork ban is
  contextual (fork `WITHHELD` tier + interactive CAS baton) and does not transfer.
  Residual: confirm `primary_worktree`'s bare-repo / not-a-repo error path suits
  the conformance + record-delta call sites.

## Open questions (remaining)

- **OQ-conf-1** — *Resolved by construction (D5 deterministic capture).* The solo
  arm no longer passes `--start`/`--end` on the happy path: `code_start = HEAD` at
  the `slice phase … in_progress` transition, `code_end = HEAD` at `… completed`.
  The phase-base question dissolves — there is no ref to choose, only HEAD at two
  well-defined lifecycle moments. (`record-delta`'s explicit `--start`/`--end`
  survive for the manual-fallback path only.)
- **OQ-conf-2** — does `record-delta` belong under `slice` or a neutral verb both
  arms share without the `slice` prefix? Lean `slice record-delta` for v0.1.
- **OQ-conf-3** — *Resolved → (a), reusing `primary_worktree` (D5/R5/F-5).* The
  resolver's bare-repo / not-a-repo error path is confirmed at the call sites in
  `/plan`.

## References

- RFC-004 — v0.1 scope, OQ-5/OQ-11/OQ-11a (settled), the accretion model.
- POL-002 — platform independence; recorded SHAs over `(SL-NNN)` grep.
- ADR-006 / ADR-012 — one source-delta commit per phase; preserved code branches.
- ADR-007 §D-C10 — `domain_map` warm-cache (the burned anchor).
- ADR-001 — module layering (leaf ← engine ← command, no cycles).
- ADR-004 / ADR-010 — relations / unified contract (the deferred target sum type).
