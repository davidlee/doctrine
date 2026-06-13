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
- **Behaviour-preservation.** SL-042/044 read+drift suites and the conduct suite
  stay green; the engine invariants (4-tuple key, `drift` matrix, sole-writer) are
  untouched.
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
  │                       ONE run per argv (Command); per-entry matcher eval; derive_status; re-stamp git::head_sha; save
  └───────────────────────────────────────────────────────┘
        │ writes coverage.toml (observed)         × never authored requirement status (NF-001)
```

### 5.2 Interfaces & Contracts

**Verification contract (`doctrine.toml`, root — opt-in, tolerant):**

```toml
[verification]
command        = ["cargo", "test"]   # project-default base argv (optional)
default-source = "stdout"            # default matcher source (optional)

[verification.aliases]
unit = ["cargo", "test"]             # name -> base argv
e2e  = ["pnpm", "test:e2e"]
```

**VT check on a coverage entry** — runnable = `base ++ extra_args`, where `base` =
`aliases[alias]` | literal `command` | project-default `command`:

```toml
[[entry]]
mode    = "VT"
# (a) default base, matcher REQUIRED:
matcher = { source = "stdout", pattern = "coverage::upsert .* ok" }
# (b) explicit literal command, matcher OPTIONAL:
# command = ["cargo", "test"]
# extra-args = ["coverage::upsert"]
# (c) alias, matcher REQUIRED (shared base doesn't prove your case):
# alias = "unit"
# matcher = { pattern = "" }   # empty == conscious exit-code-only opt-out
```

**Verdict (pure fold over a shell-produced outcome):**

| outcome | status |
|---|---|
| couldn't run (unknown alias / no runnable base / spawn fail / unreadable matcher source) | `Blocked` |
| ran, `exit 0` AND (matcher matches, when present) | `Verified` |
| ran, otherwise (`exit≠0` or matcher miss) | `Failed` |

**Validity (record-time, fail-fast) — two composed checks (F-1):**
- `coverage::valid(&VtCheck)` (pure, config-free): `mode ∈ {VT,VA,VH}`; `alias ⊕
  command` (never both); **matcher mandatory unless explicit `command`**; matcher
  regex parses.
- `verify::resolve(cfg, check)` (needs config): a runnable base actually resolves
  (`alias` exists / a default `command` is present) — else `ResolveError`.

The `record` shell runs both before writing; `valid` alone cannot judge base
resolution (it has no config). Malformed matcher regex is rejected here, not left
to surface as a `Failed`/`Blocked` at verify.

### 5.3 Data, State & Ownership

New, additive types in `coverage.rs` (pure leaf):

```rust
struct VtCheck {                      // additive Option<VtCheck> on CoverageEntry
    alias: Option<String>,            // XOR command
    command: Option<Vec<String>>,     // explicit literal argv
    extra_args: Vec<String>,          // appended onto resolved base
    matcher: Option<Matcher>,         // required unless `command` set
}
struct Matcher {
    source: Option<MatchSource>,      // None => project default => baked Stdout
    pattern: String,                  // regex_lite; "" => always matches (opt-out)
}
enum MatchSource { Stdout, Stderr, File(String /* glob */) }

enum RunOutcome { Unobtainable, Ran { exit_ok: bool, matched: Option<bool> } }
fn derive_status(&RunOutcome) -> CoverageStatus       // the verdict table (pure)
fn evaluate_matcher(pattern, haystack) -> Option<bool> // None = unparseable pattern; empty => Some(true); else regex_lite
fn valid(&VtCheck) -> Result<(), …>                   // XOR + matcher rule + regex-parses (pure; NOT base resolution — F-1)
```

`verify.rs` (leaf): `VerificationConfig { command, default_source, aliases }` +
pure `resolve(cfg, check) -> Resolved { argv, source } | ResolveError
{BothAliasAndCommand, UnknownAlias, NoRunnable}`.

**Ownership.** `coverage.toml` (observed tier) is the *only* file the write path
mutates, keyed to the owning slice. `verify` changes only `status` + `git_anchor`;
the 4-tuple key, `touched_paths`, and `check` are preserved through `upsert`.
Authored requirement status is never read or written here.

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
   each distinct `argv` **once** (`Command::new(argv[0]).args(&argv[1..]).output()`).
   Per-slice is the *write* unit (§5.3); dedup spans the whole invocation, or a
   project whose default is `cargo test` would run the full suite once per slice.
4. Per entry: build `RunOutcome` — `exit_ok` from `status.success()`; for a matcher,
   `evaluate_matcher(&m.pattern, captured(source))` → `Some(bool)`, or `None` for an
   **unparseable hand-edited pattern** ⇒ `Unobtainable` ⇒ `Blocked` (a config error,
   not a test contradiction — F-3). `source` text is stdout/stderr from the captured
   `Output`, or a `File(glob)` read from disk (unreadable/no-match ⇒ `Unobtainable`).
5. `upsert(status = derive_status(outcome), git_anchor = head)`; preserve key /
   `touched_paths` / `check`.
6. `coverage_store::save(root, slice, &file)`. Report per entry (`key: old → new`)
   plus the loud **"N VT entries lack a check — backfill"** line.

**Re-stamp HEAD on every verify** — "last observed at HEAD"; staleness resets to
Fresh after a green verify. This *is* the continuous re-derivation.

**Transient-verification resolution map** (so nothing becomes noisy-unresolvable):
- whole-requirement transient → `Retire`/`Supersede` the requirement → `drift` =
  Coherent (existing).
- cell-level transient on a live requirement → `coverage forget <key>` (new) — a
  `Failed`/`Blocked` cell otherwise poisons the live req's composite forever.
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
- **Edge — empty matcher pattern.** Trivially satisfied ⇒ verdict on exit alone.
- **Edge — malformed regex.** Rejected at record-time validity (fail-fast); a
  hand-edited one surviving into `coverage.toml` ⇒ `Blocked` at verify, never a
  silent `Failed` (F-3).
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
- **OQ-6.** `File(glob)` matching multiple files — search the concatenation, any-match,
  or reject ambiguity? → `/plan` / impl.

## 7. Decisions, Rationale & Alternatives

- **D1 — check identity = base argv (alias|literal|default) + extra_args + optional
  matcher.** Polyglot, project-agnostic. *Alt rejected:* literal command per entry
  (couples the observed tier to a toolchain); single project-wide command (can't
  target a requirement; vacuous).
- **D2 — shared single `doctrine.toml` reader (`dtoml.rs`); domain sub-configs owned
  by their modules.** *Alt rejected:* a second independent parser (the parallel
  reader CLAUDE.md forbids); bolting `verification` onto `conduct`'s struct (poor
  cohesion).
- **D3 — verdict = `exit 0 ∧ matcher`; matcher mandatory unless explicit `command`.**
  Kills vacuous satisfaction while keeping single-lang bookkeeping light via a
  project-default command + default source. *Alt rejected:* unconditional matcher
  (heavy); exit-code-only (vacuous-pass trap).
- **D4 — `coverage` becomes a subcommand group (`show`/`record`/`verify`/`forget`).**
  clap can't disambiguate a bare positional `<REFERENCE>` from subcommand names.
  Conscious golden/skill churn over a shim.
- **D5 — `verify` writes per-slice (one file); `--all` sweeps.** A requirement spans
  slices but writes go to the owning slice's file → per-slice is the coherent write
  unit. *Alt rejected:* verify-by-requirement (writes scatter across many files).
- **D6 — `forget` (removal), not a withdrawn `CoverageStatus`.** Resolves the
  transient-cell case without perturbing the shared `drift`/`composite` folds.
- **D7 — no compat jank for legacy entries.** Surface backfill loudly; update
  affected tests consciously. *Per user directive.*

## 8. Risks & Mitigations

- **R1 — verifier launders authored status (NF-001 breach).** *Mitigation:* INV-1
  guard drives `run()` end-to-end (not a pure helper — the
  `invariant-test-must-drive-the-write-seam` lesson, SL-044/RV-004 F-1), with a
  non-vacuity guard.
- **R2 — the shared `dtoml` refactor regresses conduct.** *Mitigation:* `conduct::parse`
  kept as a delegate; conduct suite green unchanged is the proof.
- **R3 — arbitrary subprocess execution.** The verifier runs project-config commands.
  *Mitigation:* commands come only from the committed root `doctrine.toml`
  (`[verification]`), never from per-entry free-text in untrusted corpus data; argv
  is a list (no shell splitting).
- **R4 — clippy ceilings on the many-flag `record` handler.** *Mitigation:* args
  struct, not N params (`cli-handler-args-struct`).

## 9. Quality Engineering & Validation

Evidence the slice produces — see also the requirements proposed below (minted +
wired as `requirements` rows at `/plan`):

- **`coverage.rs` (VT):** `derive_status` truth table; `evaluate_matcher` (empty,
  match, miss, bad-regex); `valid` reject matrix; `VtCheck`/`Matcher` round-trip +
  additive parse of a pre-SL-057 entry.
- **`verify.rs` (VT):** tolerant parse; alias resolution; both-base reject; default
  fallback; source precedence (entry → default → Stdout).
- **`coverage_verify` (VT):** dedup (one argv → one run, N entries); exit/matcher →
  status; spawn-fail / unknown-alias ⇒ `Blocked`; HEAD re-stamp; key/`touched_paths`
  preserved; check-less VT reported + untouched.
- **NF-001 guard (VA/VH framing, structural):** drive `run()` end-to-end, vary run
  outcomes, assert the requirement entity's authored status on disk is unchanged and
  only `coverage.toml` mutated.
- **CLI (VT, black-box goldens):** `record` happy-path + each validity reject;
  `verify`/`forget` surface; relocated `show`.
- **Behaviour-preservation:** SL-042/044 read+drift + conduct suites green;
  consciously-updated bare-`coverage` → `coverage show` goldens.
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

**Pending external pass:** offer `/inquisition` or an external adversarial reviewer
(codex / Opus sub-agent) before lock — see next step.
