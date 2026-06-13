# Design SL-057: Formal VT verification: executable check + coverage record surface (SPEC-002 test-run surface)

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Build the **test-run surface** SPEC-002 deferred (its "Contracts deferred"
concern): turn a VT ("by test") coverage entry from a hand-set *attestation* into
a *continuously re-derived verification*. Two gaps motivate it:

1. **No production write path.** Coverage entries are minted only by test helpers
   (`fs::write` of `[[entry]]` TOML); the design's "coverage written at audit"
   surface was never built. In production `coverage.toml` is hand-authored —
   SPEC-002 itself was shown "met" by hand-authoring entries.
2. **VT is attestation, not verification.** A VT entry carries a `git_anchor` +
   `touched_paths` + a hand-set `status`. It names no test and nothing re-runs it;
   `Verified` is asserted, not checked.

Ship: a coverage **write/withdraw** path, a **runnable check identity** on VT
entries, and an impure **verifier** that derives observed `CoverageStatus` from a
real command run — while preserving the two-tier wall and `reconcile`-sole-writer
invariant (NF-001).

## 2. Current State

- **`coverage.rs`** (pure leaf, ADR-001) — `CoverageKey` (the 4-tuple
  `slice × requirement × contributing_change × mode`), `CoverageEntry`
  (`status`/`git_anchor`/`attested_date?`/`touched_paths`), the `upsert` no-clobber
  fold, `parse`/`render`, and the `composite`/`drift` read folds. The write helpers
  (`upsert`/`parse`/`render`/`CoverageEntry`) are **test-only** — the module carries
  a `#![cfg_attr(not(test), expect(dead_code, …))]` because P2 built them ahead of a
  consumer.
- **`coverage_scan.rs`** (impure) — the *only* current git/disk seam in the coverage
  flow: corpus-walks every slice's `coverage.toml`, resolves staleness, hands the
  pure folds in-memory cells. Read-only.
- **`coverage_view.rs`** — the read-only `doctrine coverage <REFERENCE>` drift view.
- **`reconcile.rs`** — sole author of *authored* requirement/spec truth (SL-044).
- **`conduct.rs`** — prior art for the project config seam: a *private*
  `DoctrineToml` parsing only `[conduct]` from root `doctrine.toml`, tolerant of
  every other table; pure `parse` + `resolve` (per-field precedence), thin-shell
  read. Root `doctrine.toml` does **not** exist in this repo (baked defaults in
  force); reference template at `install/doctrine.toml.example`.
- **`git.rs`** — the subprocess pattern (`run_git`: `Command::new().output()` →
  `.status.success()`) and the HEAD/staleness seam.
- `CoverageStatus = {Planned, InProgress, Verified, Failed, Blocked}` (no withdrawn
  variant). `ReqStatus = {Pending, InProgress, Active, Deprecated, Retired,
  Superseded}`; `drift` already forces **Coherent** for `Retired|Superseded`.

## 3. Forces & Constraints

- **Two-tier wall (NF-001 / `REQ-105` / ADR-009 §3).** No function maps coverage →
  authored requirement status. The verifier writes the **observed** tier only;
  `reconcile` stays the sole author of authored truth.
- **Pure/imperative split (ADR-001, CLAUDE.md).** No clock/rng/git/disk/proc in the
  pure layer. The verdict (`exit/matcher → status`) is a pure fold; only the
  subprocess run + file IO + HEAD read are impure (mirrors `git.rs`).
- **Project-agnostic (ADR-011 spirit).** Nothing in the engine is cargo/Rust-specific;
  the concrete command is project config.
- **No parallel implementation (CLAUDE.md).** Ride `coverage.rs` folds, the
  `coverage_scan` staleness seam, the `git.rs` subprocess pattern, and the
  `conduct.rs` config-seam pattern. One `doctrine.toml` reader, not two.
- **No compat jank** (user directive). Legacy/check-less entries are surfaced as a
  loud backfill signal, never papered over. Conscious test churn (e.g. relocated
  CLI goldens) is preferred to backward-compat shims.
- **Behaviour-preservation (scoped — F-V).** Two distinct claims, not one. (a) The
  **engine/fold invariants** — 4-tuple key, `upsert` no-clobber, the `drift` matrix,
  `composite`, `reconcile`-sole-writer, and `conduct::parse` — stay green
  **byte-unchanged**; *that* is the preservation gate (CLAUDE.md). (b) The
  **CLI-surface goldens** churn **consciously** under D4 (bare `coverage` →
  `coverage show`); they are *not* part of the preservation proof. Do not conflate:
  a zero-test-edit run is the gate only for (a).
- **Clippy denies** (repo): `var_os` not `env::var`; `BTreeMap` not `HashMap`; no
  `expect`/`unwrap` in non-test; `Vec`-concat string assembly; `expect`+reason not
  `allow`.

## 4. Guiding Principles

- **Execution is the load-bearing proof.** Only running a test proves it is *wired*
  (not commented out / `#[ignore]`'d / behind a disabled `cfg` / unregistered). A
  static glob-for-a-string cannot. So the run is the irreducible value; the matcher
  is cheap polish atop it (`regex-lite`/`glob` already in-tree).
- **No vacuous satisfaction.** A *suite running* ≠ *your case verified*. A shared or
  default command therefore demands a matcher; only an entry-authored literal
  command earns the optional-matcher exemption (an empty matcher is the conscious
  exit-code-only opt-out).
- **Continuous re-derivation, no stored-SHA trust.** Verdict is per-run, derived
  from current reality; a flappy red flips coverage. Re-running re-stamps the anchor.
- **Single responsibility.** `record` *declares* a check; `verify` *runs* it;
  `forget` *withdraws* it. The pure verdict is testable without a subprocess.

## 5. Proposed Design

### 5.1 System Model

```text
  doctrine CLI
    coverage show   <ref>                 (existing read-only drift view, relocated)
    coverage record <key + check>         declare/upsert an entry        ┐
    coverage verify <slice> [--all]       run resolved checks, re-derive  │ write the
    coverage forget <key>                 withdraw an entry               ┘ OBSERVED tier only
        │
  ┌─────┴───────────────────────────────────────────────┐
  │ verify.rs (leaf)      VerificationConfig + pure resolve(cfg, check) -> argv + source
  │ coverage.rs (leaf)    VtCheck/Matcher types; pure derive_status, evaluate_matcher, valid
  │ coverage_store.rs     impure: load/save ONE slice coverage.toml; record/forget = load→upsert|retain→save
  │                       (NEW — nothing writes coverage.toml today; render + atomic tempfile write, entity.rs precedent)
  │ coverage_verify.rs    impure shell: read doctrine.toml; dedup by argv (GLOBAL over the invocation);
  │                       ONE run per argv (Command, cwd=root, timeout→Blocked); per-entry matcher eval;
  │                       derive_status; re-stamp git::head_sha ONLY on a Ran outcome; save
  └───────────────────────────────────────────────────────┘
        │ writes coverage.toml (observed)         × never authored requirement status (NF-001)
```

### 5.2 Interfaces & Contracts

**Verification contract (`doctrine.toml`, root — opt-in, tolerant):**

```toml
[verification]
command        = ["cargo", "test"]   # project-default base argv (optional)
default-source = "stdout"            # default matcher source (optional)
timeout-secs   = 300                 # per-run wall-clock cap (optional; baked default 300)

[verification.aliases]
unit = ["cargo", "test"]             # name -> base argv
e2e  = ["pnpm", "test:e2e"]
```

**VT check on a coverage entry** — runnable = `base ++ extra_args`, where `base` =
`aliases[alias]` | literal `command` | project-default `command`. The matcher rule
follows the **D3 recast (A)**: a *shared/derived* base (project-default `command`,
`alias`) cannot prove *your* case, so it MUST carry a non-empty matcher;
exit-code-only is permitted ONLY on an entry-local literal `command` (a conscious,
flagged opt-out):

```toml
[[entry]]
mode    = "VT"
# (a) default base — matcher REQUIRED (shared base, non-empty pattern).
#     Default is a LITERAL SUBSTRING (D8) — no regex-escaping of `.`/`(`:
matcher = { source = "stdout", pattern = "test result: ok. 5 passed" }
#     opt into regex with `regex = true`:
# matcher = { source = "stdout", pattern = "coverage::\\w+ .* ok", regex = true }
# (b) explicit literal command — matcher OPTIONAL (exit-code-only; the verifier
#     REPORTS this cell as `exit-code-only`, never silently trusts it):
# command = ["cargo", "test"]
# extra-args = ["coverage::upsert"]
# (c) alias — matcher REQUIRED, non-empty (shared base; the empty-pattern opt-out
#     is REJECTED here — it lives only with a literal `command`):
# alias = "unit"
# matcher = { source = "stdout", pattern = "coverage::upsert ... ok" }
```

**Verdict (pure fold over a shell-produced outcome):**

| outcome | status |
|---|---|
| couldn't run (unknown alias / no runnable base / spawn fail / **timeout** / unreadable-or-unparseable matcher source) | `Blocked` |
| ran, `exit 0` AND (matcher matches, when present) | `Verified` |
| ran, otherwise (`exit≠0` or matcher miss) | `Failed` |

A **timeout** (wall-clock > `[verification].timeout-secs`, baked default 300) is a
*couldn't-run*, not a `Failed` — the command never returned a verdict, so it yields
`Blocked` (INV-3 holds: never defaulted-green).

**Validity (record-time, fail-fast) — two composed checks (F-1):**
- `coverage::valid(&VtCheck)` (pure, config-free): `mode ∈ {VT,VA,VH}`; `alias ⊕
  command` (never both); the **D3-recast matcher rule (A)** — a *non-empty* matcher
  is mandatory unless an entry-local literal `command` is set; an empty/absent
  matcher on an `alias` or the project-default base is **rejected**, and an empty
  pattern is legal *only* alongside a literal `command`; **when `regex = true` (D8)
  the pattern must parse as `regex_lite`** (a substring pattern never fails to parse,
  so the parse check is regex-mode-only); and the **glob-confinement rule (F-III)** —
  a `File(glob)` source must be repo-tree
  relative: absolute paths and any `..` ascent above `root` are rejected here.
- `verify::resolve(cfg, check)` (needs config): a runnable base actually resolves
  (`alias` exists / a default `command` is present) — else `ResolveError`.

The `record` shell runs both before writing; `valid` alone cannot judge base
resolution (it has no config). Malformed matcher regex, an empty matcher on a shared
base, and an escaping `File` glob are all rejected here — never left to surface as a
`Failed`/`Blocked` at verify.

### 5.3 Data, State & Ownership

New, additive types in `coverage.rs` (pure leaf):

```rust
struct VtCheck {                      // additive Option<VtCheck> on CoverageEntry
    alias: Option<String>,            // XOR command
    command: Option<Vec<String>>,     // explicit literal argv
    extra_args: Vec<String>,          // appended onto resolved base
    matcher: Option<Matcher>,         // D3/A: required (non-empty) unless literal `command`
}
struct Matcher {
    source: Option<MatchSource>,      // None => project default => baked Stdout
    pattern: String,                  // "" (always-matches) LEGAL only with literal command
    regex: bool,                      // D8: false (default) => literal substring; true => regex_lite
}
enum MatchSource { Stdout, Stderr, File(String /* glob; repo-tree-relative, no abs/.. — F-III */) }

enum RunOutcome { Unobtainable, Ran { exit_ok: bool, matched: Option<bool> } }
fn derive_status(&RunOutcome) -> CoverageStatus       // the verdict table (pure)
fn evaluate_matcher(pattern, regex, haystack) -> Option<bool>
//   substring (regex=false): Some(haystack.contains(pattern)) — empty => Some(true); NEVER None (always parses)
//   regex (regex=true): None if unparseable; else Some(re.is_match) — empty => Some(true) (regex_lite)
// matched: None inside Ran => no matcher present => the shell REPORTS the cell as
// `exit-code-only` (D3/A); derive_status treats it as "matched" for the exit verdict.
fn valid(&VtCheck) -> Result<(), …>                   // XOR + D3/A matcher rule + glob-confinement
                                                       // + regex-parses (pure; NOT base resolution — F-1)
```

`verify.rs` (leaf): `VerificationConfig { command, default_source, aliases }` +
pure `resolve(cfg, check) -> Resolved { argv, source } | ResolveError
{BothAliasAndCommand, UnknownAlias, NoRunnable}`.

**Ownership.** `coverage.toml` (observed tier) is the *only* file the write path
mutates, keyed to the owning slice. `verify` changes only `status` + `git_anchor`;
the 4-tuple key, `touched_paths`, and `check` are preserved through `upsert`.
Authored requirement status is never read or written here.

**Date seam (F-VI — no hidden clock, ADR-001).** A VA/VH `record` stamps
`attested_date`; the clock is **injected**, not read in the write path — the
`record` shell takes `today: Date` (the existing date/uid pattern) with an optional
`--attested-date` CLI override for backfill. VT `record` leans `Planned` with no
`attested_date` (OQ-2); `verify` derives the real status and re-stamps `git_anchor`
(not `attested_date` — that axis is VA/VH attestation, not VT execution).

**Config reader (decision D2 — shared single reader).** Promote a `dtoml.rs`
owning the outer `DoctrineToml { conduct: ConductConfig, verification:
VerificationConfig }` + one `parse(text)`. `conduct::parse` becomes a thin delegate
(`Ok(dtoml::parse(t)?.conduct)`) so every existing conduct test/caller is untouched.
No second `doctrine.toml` parser.

### 5.4 Lifecycle, Operations & Dynamics

`coverage_verify::run(root, slice)`:

1. `cfg = dtoml::parse(read doctrine.toml)?.verification` (absent ⇒ default ⇒
   alias/default-base checks resolve to `Blocked`).
2. `file = coverage_store::load(root, slice)`; `head = git::head_sha(root)`
   (the resolver `coverage_scan` already uses — `None` on unborn/non-repo HEAD).
3. **Dedup (global over the invocation — F-2):** group *all* runnable VT entries
   being verified — across every slice swept by `--all` — by resolved `argv`; run
   each distinct `argv` **once**, **with `cwd = root`** (so `cargo test` and a
   relative `File(glob)` are deterministic — F-VII) and a **wall-clock cap**
   (`[verification].timeout-secs`, baked default 300): expiry ⇒ kill ⇒ `Unobtainable`
   ⇒ `Blocked`. Per-slice is the *write* unit (§5.3); dedup spans the whole
   invocation, or a project whose default is `cargo test` would run the full suite
   once per slice.
4. Per entry: build `RunOutcome` — `exit_ok` from `status.success()`; for a matcher,
   `evaluate_matcher(&m.pattern, captured(source))` → `Some(bool)`, or `None` for an
   **unparseable hand-edited pattern** ⇒ `Unobtainable` ⇒ `Blocked` (a config error,
   not a test contradiction — F-3). `source` text is stdout/stderr from the captured
   `Output`, or a `File(glob)` read from disk: the glob is **resolved under `root`**
   (absolute/`..`-escaping globs were rejected at record-`valid`, F-III), and its
   matches are **searched as one concatenation — any-match** (OQ-6 resolved); empty
   match-set or unreadable ⇒ `Unobtainable`.
5. `upsert(status = derive_status(outcome), git_anchor = …)`; **re-stamp `git_anchor
   = head` ONLY on a `Ran{..}` outcome — a `Blocked`/`Unobtainable` cell keeps its
   prior anchor (F-VIII)**, so staleness still bites a never-observed cell. Preserve
   key / `touched_paths` / `check`.
6. `coverage_store::save(root, slice, &file)`. Report per entry (`key: old → new`),
   flagging any **`exit-code-only`** cell (literal `command`, no matcher — D3/A) so
   it is auditable, plus the loud **"N VT entries lack a check — backfill"** line.

**Re-stamp HEAD on every *ran* verify** — "last observed at HEAD"; staleness resets
to Fresh after a verify that actually executed (`Verified`/`Failed`). A `Blocked`
cell was *not* observed, so its anchor does not advance (F-VIII). This *is* the
continuous re-derivation — honest about what was and was not seen.

**Transient-verification resolution map** (so nothing becomes noisy-unresolvable):
- whole-requirement transient → `Retire`/`Supersede` the requirement → `drift` =
  Coherent (existing).
- cell-level transient on a live requirement → `coverage forget <key>` (new) — a
  `Failed`/`Blocked` cell otherwise poisons the live req's composite forever.
  **Accountable (F-IV):** `forget` emits a loud **evidence-withdrawal** line naming
  the 4-tuple key and the status erased (`withdrew SL-…/REQ-…/…/VT [Failed]`), so a
  deletion that flips a composite green is never silent. It is the *sanctioned*
  transient-cell tool, but its use is visible. (Whether closure should additionally
  gate on a live `Failed` coverage cell — i.e. forbid "delete the red to close" — is
  out of SL-057 scope; see R5 + RSK-008.)
- VA/VH transient → decays via the existing staleness seam (stale-verified ⇒
  `Indeterminate`, not `Divergent`); SL-057 does not *run* VA/VH.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (NF-001).** No code path reads/writes authored requirement status from the
  verifier or write path. Guarded by a test that drives `run()` end-to-end.
- **INV-2.** Dedup runs each distinct `argv` exactly once regardless of entry count.
- **INV-3.** `derive_status` never returns `Verified` for an unobtainable run
  (no defaulted-green; SPEC-002 failure-mode).
- **Edge — check-less legacy VT.** Not run; status left untouched; reported as
  backfill-needed (no auto-`Blocked`, which would manufacture noise; no shim).
- **Edge — empty matcher pattern.** Trivially satisfied ⇒ verdict on exit alone —
  but **legal only alongside a literal `command`** (D3/A); on an `alias`/default base
  it is rejected at record-`valid`. Such cells are reported `exit-code-only`.
- **Edge — malformed regex (regex-mode only, D8).** A `regex = true` pattern is
  parse-checked at record-time validity (fail-fast); a hand-edited bad one surviving
  into `coverage.toml` ⇒ `Blocked` at verify, never a silent `Failed` (F-3). A
  substring pattern (default) has no malformed case.
- **Edge — escaping `File` glob (F-III).** An absolute or `..`-ascending glob is
  rejected at record-`valid`; a hand-edited one surviving into `coverage.toml`
  resolves under `root` and finds nothing outside it ⇒ `Unobtainable` ⇒ `Blocked`.
  The verifier never reads a host file above the repo tree.
- **Edge — timeout (F-VII).** A command exceeding `timeout-secs` is killed ⇒
  `Unobtainable` ⇒ `Blocked` (never a `Failed`, never a hang).
- **Edge — `record` covers all modes (F-4).** `record`/`forget` are the general
  observed-tier write/withdraw path (VT/VA/VH); the check + `verify` machinery
  applies to **VT only**. A VA/VH record carries `status` + `attested_date`, no
  `check`; SL-057 does not run them.
- **Assumption.** `coverage.toml` parse stays tolerant/additive — pre-SL-057 entries
  without `check` still parse (the `touched_paths` precedent), so `coverage_scan`
  /drift never break on old files.

## 6. Open Questions & Unknowns

- **OQ-1.** Key-on-CLI ergonomics: four flags (`--slice/--requirement/--change/--mode`)
  vs a compact `SL/REQ/SL/VT` token. Ergonomics, not architecture → `/plan`.
- **OQ-2.** Default VT `status` at record (lean `Planned` — intent declared, `verify`
  derives the real one) → `/plan`.
- **OQ-3.** Whether `record` auto-runs once after writing a VT check (lean **no** —
  `record` declares, `verify` runs; single responsibility).
- **OQ-4 (deferred).** Per-cell coverage withdrawal-*status* (vs `forget` removal) —
  only if `forget` proves insufficient; would perturb the `drift` matrix, so avoided
  for now.
- **OQ-5.** Is `touched_paths` meaningful on a VT check? `verify` re-stamps
  `git_anchor = HEAD` ⇒ staleness is Fresh immediately after, so `touched_paths`
  only bites *between* verifies. Lean optional for VT → `/plan`.
- **OQ-6 (RESOLVED — F-VII).** `File(glob)` matching multiple files: search the
  **concatenation of all matches (any-match)**; empty match-set ⇒ `Unobtainable` ⇒
  `Blocked`. Resolved in design (verdict-affecting — not deferrable to impl); glob is
  repo-tree-confined (F-III).

## 7. Decisions, Rationale & Alternatives

- **D1 — check identity = base argv (alias|literal|default) + extra_args + optional
  matcher.** Polyglot, project-agnostic. *Alt rejected:* literal command per entry
  (couples the observed tier to a toolchain); single project-wide command (can't
  target a requirement; vacuous).
- **D2 — shared single `doctrine.toml` reader (`dtoml.rs`); domain sub-configs owned
  by their modules.** *Alt rejected:* a second independent parser (the parallel
  reader CLAUDE.md forbids); bolting `verification` onto `conduct`'s struct (poor
  cohesion).
- **D3 (recast A — anti-vacuity, F-I/F-II) — verdict = `exit 0 ∧ matcher`; a
  non-empty matcher is mandatory on any *shared/derived* base (project-default
  `command`, `alias`), and exit-code-only is permitted ONLY on an entry-local
  literal `command`, where the verifier *reports* the cell as `exit-code-only`.**
  The original "matcher unless explicit `command`" left a hole: a literal
  `command = ["cargo","test"]` (or an empty matcher on an alias) ran the whole suite
  and minted `Verified` — the vacuous-pass §4 forbids. The recast enforces structure
  where the framework *can* prove vacuity (a shared base cannot target your case) and
  makes the residual (a hand-authored literal command) a conscious, **flagged**,
  auditable act rather than a silent copy-paste. *Alt rejected:* unconditional matcher
  on every base (heavy; a bespoke fail-on-absence command must carry redundant
  bookkeeping — option C); `extra_args`-narrowing as proof (option B — a filter
  matching zero tests still exits 0, so narrowing ≠ execution).
- **D4 — `coverage` becomes a subcommand group (`show`/`record`/`verify`/`forget`).**
  clap can't disambiguate a bare positional `<REFERENCE>` from subcommand names.
  Conscious golden/skill churn over a shim.
- **D5 — `verify` writes per-slice (one file); `--all` sweeps.** A requirement spans
  slices but writes go to the owning slice's file → per-slice is the coherent write
  unit. *Alt rejected:* verify-by-requirement (writes scatter across many files).
- **D6 — `forget` (removal), not a withdrawn `CoverageStatus`.** Resolves the
  transient-cell case without perturbing the shared `drift`/`composite` folds.
  **Accountable (F-IV):** `forget` is loud — it prints an evidence-withdrawal line
  naming the key + erased status — so deleting a red cell is never silent. The
  stronger guard (closure refusing while a live `Failed` cell exists) is deferred
  out of scope (R5 + backlog), not adopted, to avoid perturbing the close-gate here.
- **D7 — no compat jank for legacy entries.** Surface backfill loudly; update
  affected tests consciously. *Per user directive.*
- **D8 — matcher is a literal substring by default; regex is opt-in (`regex = true`).**
  A plain `pattern` matches by `haystack.contains(pattern)`, so test output
  containing regex metacharacters (`.`, `(`, `+`) needs no escaping — the common
  case. `regex = true` selects `regex_lite` for the power case. *Rationale:* least
  surprise + least typing for the dominant "this literal line appears" intent; no
  compat cost (zero production check entries pre-SL-057). *Alt rejected:* regex-only
  (forces escaping ordinary punctuation — the prior shape); XOR `regex`/`substring`
  keys (heavier than one bool for a binary mode).

## 8. Risks & Mitigations

- **R1 — verifier launders authored status (NF-001 breach).** *Mitigation:* INV-1
  guard drives `run()` end-to-end (not a pure helper — the
  `invariant-test-must-drive-the-write-seam` lesson, SL-044/RV-004 F-1), with a
  non-vacuity guard.
- **R2 — the shared `dtoml` refactor regresses conduct.** *Mitigation:* `conduct::parse`
  kept as a delegate; conduct suite green unchanged is the proof.
- **R3 — arbitrary subprocess execution + arbitrary file read.** The verifier runs
  project-config commands *and* reads `File(glob)` matcher sources. *Mitigation:*
  **commands** come only from the committed root `doctrine.toml` (`[verification]`),
  never per-entry; argv is a list (no shell splitting). The **matcher `File` source**
  *is* per-entry corpus free-text (F-III) — so it is confined: the glob is
  repo-tree-relative, absolute/`..`-escaping globs rejected at record-`valid`,
  resolution rooted at `root`. No host file above the repo is ever read.
- **R4 — clippy ceilings on the many-flag `record` handler.** *Mitigation:* args
  struct, not N params (`cli-handler-args-struct`).
- **R5 (deferred, F-IV) — `forget` as closure-evasion.** Deleting a `Failed` cell to
  flip a composite green is *visible* (D6 loud line) but not *prevented*; closure
  does not yet gate on a live `Failed` coverage cell. *Disposition:* out of SL-057
  scope (would perturb the close-gate); captured as RSK-008. Accepted
  residual: an auditor can see the withdrawal, but tooling does not block it.

## 9. Quality Engineering & Validation

Evidence the slice produces — see also the requirements proposed below (minted +
wired as `requirements` rows at `/plan`):

- **`coverage.rs` (VT):** `derive_status` truth table (incl. timeout/`Unobtainable`
  ⇒ `Blocked`, F-VII); `evaluate_matcher` (D8) — substring match/miss, **metachar
  (`.`/`(`) matched literally in substring mode**, regex match/miss, `regex=true`
  bad-pattern ⇒ `None`, empty ⇒ `Some(true)` both modes; `valid`
  reject matrix — incl. **empty/absent matcher on `alias`/default ⇒ reject, empty
  matcher with literal `command` ⇒ accept (D3/A)**, and **`File` glob absolute /
  `..`-escaping ⇒ reject (F-III)**; `VtCheck`/`Matcher` round-trip + additive parse
  of a pre-SL-057 entry.
- **`verify.rs` (VT):** tolerant parse (incl. `timeout-secs`); alias resolution;
  both-base reject; default fallback; source precedence (entry → default → Stdout).
- **`coverage_verify` (VT):** dedup (one argv → one run, N entries); exit/matcher →
  status; spawn-fail / unknown-alias / **timeout** ⇒ `Blocked`; **`git_anchor`
  re-stamped on `Ran` but NOT on `Blocked` (F-VIII)**; `cwd = root` (F-VII);
  confined-glob any-match (F-VII/OQ-6); **`exit-code-only` cell flagged in the report
  (D3/A)**; key/`touched_paths` preserved; check-less VT reported + untouched.
- **`forget` (F-IV):** removes the keyed cell AND emits the evidence-withdrawal line
  naming key + erased status; a test asserts the line fires on a `Failed` cell.
- **Date seam (F-VI):** `record` of a VA/VH stamps the **injected** `today` (and the
  `--attested-date` override); a test asserts no clock is read in the write path.
- **NF-001 guard (VA/VH framing, structural):** drive `run()` end-to-end, vary run
  outcomes, assert the requirement entity's authored status on disk is unchanged and
  only `coverage.toml` mutated.
- **CLI (VT, black-box goldens):** `record` happy-path + each validity reject
  (empty-matcher-on-shared-base, escaping-glob, bad-regex, both-base);
  `verify`/`forget` surface; relocated `show`.
- **Behaviour-preservation (scoped — F-V):** (a) **the gate** — SL-042/044 read+drift
  fold suites + the conduct suite stay green **byte-unchanged** (no test edits);
  (b) **conscious churn** — the bare-`coverage` → `coverage show` CLI goldens are
  updated by hand (D4), and are explicitly *not* part of (a).
- **Dogfood closure (optional):** SL-057 records VT checks for its own requirements
  and `verify`s them green at `/close` — replacing the hand-authored backfill.

**Requirements SPEC-002 gains (lift the "Contracts deferred" line) — proposed,
authored at `/plan`:** R-a runnable check identity (FR); R-b derived status /
continuous re-derivation (FR); R-c production write/withdraw path (FR); R-d
observed-tier confinement / NF-001 reaffirmed (NF); R-e project-agnostic contract (NF).
Explicitly *not* asserted: matcher polarity, multi-line matching, per-cell
withdrawal-status.

## 10. Review Notes

**Adversarial self-review pass 1 (author, integrated above).**

- **F-1 — `valid` over-claimed (precision).** A pure `valid(&VtCheck)` cannot decide
  "a runnable base resolves" — that needs the config. *Fixed §5.2/§5.3:* split into
  `coverage::valid` (XOR + matcher rule + regex-parses, pure) and `verify::resolve`
  (base resolution); `record` composes both.
- **F-2 — `--all` would run the suite once per slice (perf/correctness).** *Fixed
  §5.1/§5.4:* dedup is **global over the invocation**; per-slice is only the *write*
  unit.
- **F-3 — bad matcher regex at verify-time.** Record rejects it, but a hand-edited
  `coverage.toml` can carry one. *Fixed §5.4/§5.5:* unparseable pattern ⇒
  `Blocked` (config error), never a silent `Failed`; `evaluate_matcher` returns
  `Option<bool>`.
- **F-4 — record mode scope ambiguous.** *Fixed §5.5:* `record`/`forget` are the
  general observed-tier write/withdraw path (VT/VA/VH); check + `verify` is VT-only.
- **F-5 — `coverage_store` is new, not a ridden write seam.** Verified: nothing writes
  `coverage.toml` today, so it is *not* a parallel-impl violation. *Noted §5.1:* model
  `save` on render + atomic tempfile write (the `entity.rs::write_fileset` precedent).
- **F-6 — `git::head_sha` confirmed** as the reusable HEAD resolver (`coverage_scan`
  uses it); `None` on unborn/non-repo HEAD. *Named §5.4.*

**Verified against source:** `git::head_sha` / `git::commits_touching` (`git.rs`,
called from `coverage_scan`); the `not(test)` dead_code expect on `coverage.rs`
write helpers (this slice retires it); `conduct.rs` private `DoctrineToml`
(the D2 delegate target); `CoverageStatus` has no withdrawn variant (D6 rationale).

**Adversarial review pass 2 — Inquisition (`inquisition.md`, integrated above).**
Eight charges, all upheld; dispositions integrated:

- **F-I/F-II (GRAVE) — anti-vacuity guard hollow.** A literal `command=["cargo","test"]`
  (or an empty matcher on an alias) minted `Verified` over the whole suite. *Fixed
  §4/§5.2/§5.3/§7-D3 (recast A):* non-empty matcher mandatory on shared/derived bases;
  exit-code-only only on a literal `command`, reported `exit-code-only`.
- **F-III (GRAVE, security) — `File(glob)` arbitrary disk read** from per-entry corpus
  data, contra R3's own trust boundary. *Fixed §5.2-validity/§5.3/§5.5/§8-R3:*
  repo-tree-confined glob; absolute/`..` rejected at record-`valid`; rooted at `root`.
- **F-IV (SERIOUS) — `forget` silent evidence erasure.** *Fixed §5.4/§7-D6:* loud
  evidence-withdrawal line; the stronger closure-gate-on-`Failed` deferred (R5 +
  backlog), not silently adopted.
- **F-V (MODERATE) — behaviour-preservation overclaim.** *Fixed §3/§9:* split into
  (a) engine/fold suites byte-green = the gate vs (b) consciously-churned CLI goldens.
- **F-VI (MODERATE) — `attested_date` hidden clock.** *Fixed §5.3:* injected `today`
  + `--attested-date`; no clock in the write path.
- **F-VII (MODERATE) — cwd/timeout/multi-match deferred.** *Fixed §5.1/§5.4/§6-OQ-6:*
  `cwd=root`; `timeout-secs`→`Blocked`; OQ-6 resolved (concatenation any-match).
- **F-VIII (MINOR) — HEAD re-stamp on un-run `Blocked`.** *Fixed §5.4/§5.5:*
  `git_anchor` advances only on a `Ran` outcome; `Blocked` keeps its prior anchor.

**Verified against source (pass 2):** `CoverageStatus` carries no `Withdrawn` (D6);
the self-retiring `not(test)` dead_code expect on `coverage.rs`; `git::head_sha`;
`conduct.rs` private `DoctrineToml` (only `[conduct]`, tolerant — D2 target); no
production `coverage.toml` writer (F-5); `regex-lite`+`glob` in-tree; the bare
`Coverage { reference }` positional D4 relocates.
