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

### D5 — Recorded source-delta registry: arm-neutral runtime state (OQ-D)

One registry both arms write and the auditor reads, resolved to a single shared
location regardless of which worktree the command runs in.

- **Home:** `.doctrine/state/slice/<NNN>/boundaries.toml` under the **primary
  working tree**, resolved via a new `root` helper (git-common-dir → primary
  worktree), *not* CWD-relative `root::find`. This closes R5: dispatch (in the
  coordination tree), solo `/execute` (in a fork), and `conformance` (in the main
  tree) all hit the same file. Worktree layout is a doctrine-owned contract
  (ADR-006/012), so this is POL-002-clean. Runtime, gitignored, in-loop lifetime.
- **Row type:** reuse ledger's `BoundaryRow { phase, code_start_oid,
  code_end_oid }` — no new parallel row type. (Shared type; if a move is needed to
  avoid a layering cycle, it relocates to a leaf module both can import.)
- **Writers:**
  - solo `/execute` → `doctrine slice record-delta` at phase land.
  - dispatch `integrate` (dispatch.rs:519/1550) → projects its already-recorded
    coordination boundaries into the arm-neutral registry as each phase lands (a
    thin read-coordination-boundaries → write-state-registry call; the one touch
    to a live dispatch path — kept minimal).
- **Reader:** `slice conformance` reads only this registry.
- **Degrade (OQ-1):** empty/absent registry → an honest `unavailable` verdict
  ("no recorded delta — audit the diff manually"). No fabricated weaker
  substitute (inline-on-main work falls back to human eyeballing).

### D6 — Slice-delta + the algebra (pure core)

```
actual : Map<path, Status(A|M|D)>
       = ⋃ over registry rows of: git diff --name-status <code_start>..<code_end>
```

Per-phase code ranges never include the merge commits that pull `main` in, so
interleaved trunk contributes nothing and **no base ref is needed** (RFC-004
OQ-11). Match git's actual paths against the **design-target** selectors by glob
(a literal path is a degenerate glob — no tree resolution needed for the diff):

```
conformant  = { p ∈ actual.keys | ∃ design-target selector matching p }
undeclared  = { p ∈ actual.keys | no design-target selector matches p }   # highest signal; carries A|M|D
undelivered = { design-target selector | no actual path matches it }
```

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
- `src/state.rs` — `boundaries_path(slice)`, `record_source_delta` writer,
  reader (reusing `BoundaryRow`).
- `src/dispatch.rs` — projection into the arm-neutral registry at `integrate`.
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
- `/execute` — call `slice record-delta` at phase land (solo arm).
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
  literal / empty-registry-degrade / multi-phase union.
- **`slice conformance` integration test** — fixture slice with a recorded
  registry + a synthetic commit range; assert the three cells.
- **`slice selector` tests** — batch add (variadic), upsert semantics, `note`
  override, `rm`, round-trip through `slice-NNN.toml`.
- **Review behaviour-preservation** — existing staleness tests stay green with
  the selector source swapped in; domain_map-input tests migrated.
- **POL-002 conformance (VH)** — reviewer challenge: no `(SL-NNN)` grep; delta
  rests only on recorded oids.
- **Prove-value (VA/VH)** — dogfood `slice conformance SL-147` against SL-147's
  own recorded boundaries before close; confirm the diff surfaces signal a human
  would otherwise hand-hunt.

## Non-goals (reaffirmed)

Per-PHASE attribution · author-declared verb sub-tags · target sum type /
non-entity edge generalization (OQ-6) · new verify mode VG (OQ-9) · prose
invariants/risks · dispatch disjointness · IMP-012 wiring · durable post-close
registry · MCP reader (fast-follow). Conformance is necessary-not-sufficient: it
says *where to look*, never *whether it passes*.

## Residual risks

- **R1 — dispatch `integrate` projection.** The one touch to a live dispatch
  path. Mitigate: keep it a thin read→write of an existing struct; gate on the
  existing dispatch tests.
- **R2 — glob `undelivered` semantics.** A forward-looking glob ("will create
  src/foo/*") that matches nothing in `actual` reads as undelivered; that is the
  correct signal (declared, not delivered), but a glob matching nothing because
  the work is *pending* (mid-slice) is noise. Mitigate: conformance is an
  audit-time (slice-complete) read; document the timing assumption.
- **R3 — `BoundaryRow` reuse across modules.** Risk of an ADR-001 layering cycle
  if `state.rs` imports `ledger`. Mitigate: relocate the row to a shared leaf if
  needed (a struct move, not a re-implementation).
- **R4 — review suite churn.** Burning the input changes test fixtures; the
  computation behaviour is the invariant, not the input format.
- **R5 — cross-worktree registry visibility (RESOLVED → option (a)).**
  `root::find` is CWD-relative with no git-common-dir awareness, and the dispatch
  orchestrator `cd`s into the coordination worktree before `sync` (dispatch-agent
  SKILL.md), so a coordination-tree `.doctrine/state/` registry is invisible to
  `conformance` in the main tree. **Resolution:** a new `root` helper resolves the
  registry against the **primary working tree** (git-common-dir → primary
  worktree), used by both `record-delta` and `conformance` (D5). Single shared
  file, arm-agnostic. The helper is reusable platform infrastructure (likely
  wanted elsewhere). Residual: the helper must handle the non-worktree / bare-repo
  edge and the "not in a repo" error path.

## Open questions (remaining)

- **OQ-conf-1** — `record-delta` ref ergonomics on the solo arm: what `--start`/
  `--end` does `/execute` pass (phase-base vs `HEAD~n`)? Resolve in `/plan` /
  `/phase-plan` against the actual solo landing sequence (this is scope-doc OQ-D's
  residue).
- **OQ-conf-2** — does `record-delta` belong under `slice` or a neutral verb both
  arms share without the `slice` prefix? Lean `slice record-delta` for v0.1.
- **OQ-conf-3** — *Resolved → (a).* Primary-working-tree resolver (D5/R5). The
  helper's edge cases (bare repo, not-a-repo) land in `/plan`.

## References

- RFC-004 — v0.1 scope, OQ-5/OQ-11/OQ-11a (settled), the accretion model.
- POL-002 — platform independence; recorded SHAs over `(SL-NNN)` grep.
- ADR-006 / ADR-012 — one source-delta commit per phase; preserved code branches.
- ADR-007 §D-C10 — `domain_map` warm-cache (the burned anchor).
- ADR-001 — module layering (leaf ← engine ← command, no cycles).
- ADR-004 / ADR-010 — relations / unified contract (the deferred target sum type).
