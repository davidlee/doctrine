# Design SL-007: Memory anchoring & capture

## 1. Design Problem

Build the **producer** that memory v1 retrieval depends on ([slice-007.md](slice-007.md)):
make `record` capture a memory's **scope** (where it lives) and a **git born
frame** (the commit it was true against), give the `Memory` parser the fields to
carry them, and add a minimal `verify` verb that advances the verification axis.
The reader (`find`/`retrieve`, ranking, staleness) is the sibling slice SL-008 and
is explicitly out of scope.

This is *not* a retrieval design. It is an **anchoring** design, and the three hard
parts are all on the write/IO side:

1. **The born frame is a locked contract, and doctrine has no git surface.** Locked
   decision 6 ([memory-spec](../../../doc/memory-spec.md) § Locked decisions) fixes
   the minimum frame: `repo + HEAD commit/tree/ref + dirty + checkout_state_id +
   base_commit`. The only `git` token in `src/` today is `.git` as a root marker
   (`src/root.rs`). This slice builds doctrine's first git seam (`src/git.rs`) and
   must implement the *whole* frame, not a convenient subset.
2. **`verified_sha` must not be written at capture.** The spec defines it as "SHA at
   last verification" and gates attested staleness solely on its presence
   (§ Retrieval). `record` verifies nothing; writing `verified_sha = HEAD` at record
   would make every memory falsely attested and turn the unattested/time-based mode
   into dead code. The capture axis (`base_commit`/`commit`, always set) and the
   verification axis (`verified_sha`, set only by `verify`) are separate; this slice
   keeps them separate.
3. **The widening is a parser change with a legacy-compat obligation.** `RawGit{}`
   is fieldless and `RawReview` parses only `verification_state` (`src/memory.rs`).
   Carrying the anchor and `reviewed` means giving these structs real fields — and
   doing it `#[serde(default)]` so the SL-005 `memory.toml` files already on disk
   (no `[git]` data, no `reviewed`) still parse.

The pure/imperative split is the spine: git subprocess calls and the clock live in
a thin shell; frame *rendering* and validation stay pure (the resolved frame is an
input).

## 2. Current State

`src/memory.rs` two-layer core + shell from SL-005:

- **Raw → validated.** `RawMemoryToml` parses every documented block, but several
  are **fieldless placeholders consumed-not-read**: `RawGit{}` (`src/memory.rs:311`),
  `RawRelation{}`, `RawSource{}`. `RawReview` (`:287`) parses **only**
  `verification_state` — there is *no* `reviewed`/`review_by` field today (so this
  slice *adds* parser fields, it does not "un-discard" them). `RawScope` (`:271`)
  already parses `paths/globs/commands/tags/workspace/repo`. `TryFrom` builds the
  validated `Memory` (`:334`) carrying the full `Scope` but dropping `[git]` and
  `reviewed`.
- **Write path.** `run_record` (`:541`) mints a v7 uid + `clock::today()`, assembles
  a `Draft` (`:447` — uid/key/type/status/title/summary/date/tags; **no scope, no
  git**), renders via `memory_scaffold` → `render_memory_toml` (`:460`, fills
  `[scope]` arrays empty, `repo=""`, from the template), and claims `items/<uid>/`
  via `entity::materialise_named`.
- **Template.** `install/templates/memory.toml` hardcodes `[scope]` empty,
  `repo=""`, `[git] anchor_kind="none"`.
- **Show framing.** `render_show` (`:595`) emits the data-block with `anchor: none`
  hardcoded and a comment naming SL-007 as the slice that fills it.
- **Mutation precedent.** `state::set_phase_status` (`src/state.rs`) and `adr status`
  (`src/adr.rs`) edit-preservingly mutate a toml via `toml_edit`. `adr status`
  targets an **authored committed** file — the exact pattern `verify` needs.
- **No git module, no `verify`, no `clock`-style git seam.**

## 3. Forces & Constraints

- **Pure/imperative split (hard):** no git or clock in the pure layer. `head_frame`
  is impure; `render_memory_toml` and validation take the resolved frame as data.
- **Locked decision 6 (hard):** the full born frame, no subset.
- **Interop constraint 4 (hard):** repo-scoped memory requires a born frame or it is
  an error; the backend never infers git — doctrine constructs the frame.
- **Storage rule:** scope + anchor are authored structured TOML (committed).
- **Behaviour-preservation:** `entity.rs` untouched, its suites green. `record`'s
  output changes intentionally; its tests update. Legacy `memory.toml` files must
  still parse (serde defaults).
- **`verified_sha` semantics (hard, from review):** capture axis ≠ verification axis.
- **No git lib dep:** subprocess `git` only.

## 4. Guiding Principles

1. **Anchor honestly.** Write what capture knows (`base_commit`, `commit`, `tree`,
   `ref`, `dirty`); never write what only verification knows (`verified_sha`).
2. **Resolve git at the edge.** One `head_frame` call per `record`; the pure render
   takes the frame as data.
3. **Every git failure is a state, not a crash.** Missing binary, non-git, unborn,
   detached → `anchor_kind = none`, surfaced explicitly.
4. **Widen additively.** New parser fields default-empty; legacy files and the bare
   `record` flow keep working.
5. **Reuse the mutation seam.** `verify` is `adr status`-shaped `toml_edit`, not a
   new mutation mechanism.

## 5. Proposed Design

### 5.1 System Model

```
record ─▶ shell: clock::today()
                 git::head_frame(root) ─▶ GitFrame (or anchor_kind=none)
                 ─▶ Draft{ scope, frame, … } ─▶ render_memory_toml (pure) ─▶ materialise_named
verify ─▶ shell: git::head_commit(root) + today ─▶ toml_edit mutate [review]+[git].verified_sha
show   ─▶ Memory.anchor ─▶ render_show (pure, real anchor line)
```

`src/git.rs` (new, impure): `head_frame`, `head_commit`, repo-identity. `src/memory.rs`:
parser widening, `Draft`/render widening, `verify` shell, `render_show` anchor line.

### 5.2 Interfaces & Contracts

**Git seam (`src/git.rs`, impure).**

```rust
pub(crate) enum AnchorKind { Commit, CheckoutState, None }

pub(crate) struct GitFrame {                 // the full locked-decision-6 frame
  pub anchor_kind: AnchorKind,
  pub repo: String,                          // normalized host/owner/name, or ""
  pub commit: String,                        // HEAD sha       (Commit)
  pub tree: String,                          // HEAD^{tree}    (Commit)
  pub ref_name: String,                      // refs/heads/... (or "" detached)
  pub dirty: bool,
  pub checkout_state_id: String,             // set iff dirty
  pub base_commit: String,                   // HEAD sha the memory sits on
}

pub(crate) fn head_frame(repo_root: &Path) -> GitFrame;   // never errors: failure → anchor_kind=None
pub(crate) fn head_commit(repo_root: &Path) -> Option<String>;  // for verify
```

`head_frame` shells: `git rev-parse --verify HEAD` (commit; failure ⇒ unborn/non-git
⇒ `None`), `git rev-parse HEAD^{tree}` (tree), `git symbolic-ref --quiet HEAD`
(ref; empty ⇒ detached), `git status --porcelain` (dirty ⇒ `CheckoutState` +
`checkout_state_id` = a hash of the porcelain output + HEAD), and the repo-identity
resolution below.

**Repo identity (locked here — review finding #8).** `git remote get-url origin`
→ normalize (`strip scheme/user@`, drop `.git`, `:`→`/`) to `host/owner/name`; no
`origin` → first remote (`git remote` first line); no remotes → `--repo` value if
given, else `""`. `repo` is a **scope coordinate and a partition key**, so it is
locked now, not deferred. Secrets never enter it (constraint: it feeds hashes on
the interop backend) — a remote URL with embedded credentials has its userinfo
stripped.

**`record` flags (new).** `--path <P>` / `--glob <G>` / `--command <C>` (each
repeatable) → the `scope` arrays; `--repo <R>` overrides the derived identity.

**`verify` verb (new).**

```
doctrine memory verify <uid|key> [-p ROOT]
```

Resolves the memory (the `resolve_show` chokepoint), reads `head_commit`, and
`toml_edit`-mutates: `[review].verification_state = "verified"`,
`[review].reviewed = today`, `[git].verified_sha = <head_commit>`, bumps `updated`.
Edit-preserving (comments/unknown keys intact). A non-git context (no `head_commit`)
verifies the review axis (`reviewed`/`verification_state`) but leaves
`verified_sha` empty — honest: there is no SHA to attest against.

**`Memory` widening.** Add `anchor: Anchor` (validated from `[git]`) and
`reviewed: String`. `Anchor` mirrors `GitFrame`'s persisted subset + `verified_sha`.

### 5.3 Data, State & Ownership

- **Parser change.** `RawGit` gains `anchor_kind/repo?/commit/tree/ref_name/dirty/
  checkout_state_id/base_commit/verified_sha`, all `#[serde(default)]`. `RawReview`
  gains `reviewed/review_by`, `#[serde(default)]`. `repo` stays in `[scope]` (its
  spec home); the anchor's git facts in `[git]`.
- **Template.** `install/templates/memory.toml` keeps its keys; `record` now
  overwrites the `[scope]`/`[git]` values from flags + frame rather than rendering
  the empty defaults. (Legacy installed templates unaffected — the change is in what
  `render_memory_toml` substitutes.)
- **Ownership.** `record` owns born-frame construction + scope capture; `verify`
  owns the verification axis; both write authored committed TOML. Nothing derived is
  stored.

### 5.4 Lifecycle, Operations & Dynamics

- **record:** resolve root → `today` + `head_frame` → validate (repo-scoped +
  `anchor_kind=none` ⇒ **error**) → `Draft` → render → `materialise_named`.
- **verify:** resolve memory → `head_commit` → `toml_edit` mutate → atomic write.
  Idempotent in effect (re-verifying at the same HEAD rewrites identical values +
  bumps `updated`/`reviewed`).
- **show:** unchanged flow; `render_show` now prints the real anchor.

### 5.5 Invariants, Assumptions & Edge Cases

- **Anchor honesty:** `verified_sha` is written by `verify` only, never by `record`.
- **Repo-scoped + unanchorable ⇒ error**, not a silent unscoped write (constraint 4).
- **Legacy parse:** an SL-005 `memory.toml` (no `[git]` fields, no `reviewed`) parses
  to `anchor_kind=none` + `reviewed=""` via serde defaults — covered by a fixture.
- **Detached HEAD / unborn / non-git:** `ref_name=""` / `anchor_kind=none`; never a
  panic.
- **Dirty tree:** `anchor_kind=checkout_state`, `checkout_state_id` set, `commit`
  still the dirty-base HEAD; `base_commit` = HEAD.
- **Secrets:** repo-identity strips URL userinfo; no credential reaches `repo`.
- **No float anywhere** (n/a here — no scores; noted for continuity).

## 6. Open Questions & Unknowns

1. **`checkout_state_id` definition.** A stable hash of `git status --porcelain` +
   `git diff` content vs just porcelain. *Lean:* hash(porcelain + HEAD) for v1 — it
   distinguishes dirty states cheaply; content-exact id deferred.
2. **`verify` in a non-git repo.** Stamp `verification_state=verified` + `reviewed`
   with empty `verified_sha` (proposed), or refuse? *Lean:* stamp the review axis;
   it is meaningful without git (the unscoped/time-based staleness mode uses it).
3. **`--repo` normalization corner cases** (SSH `git@host:owner/repo`, nested
   group paths in GitLab). *Lean:* normalize the common forms, store the raw URL
   (userinfo-stripped) as a fallback when parsing is ambiguous.

## 7. Decisions, Rationale & Alternatives

- **D1 — `record` writes the born frame but NOT `verified_sha`; a minimal `verify`
  verb stamps it.** *Rationale:* the review (BLOCKING #2) showed record-time
  `verified_sha` falsely attests every memory and kills the time-based mode; the
  spec ties `verified_sha` to verification. Capture and verification are separate
  axes. *Alternative rejected:* record-time `verified_sha` (the original D3) — spec
  deviation + dead-code. *Alternative rejected:* defer `verify` to F1 and count
  staleness from `base_commit` — needs amending the locked staleness table.
- **D2 — implement the full locked-decision-6 frame.** *Rationale:* review BLOCKING
  #3 — a subset (dropping `tree`/`dirty`) is a latent spec violation that the reader
  and the interop backend would inherit. *Alternative rejected:* minimal frame now,
  complete later — bakes a wrong contract into stored data.
- **D3 — lock repo identity now.** *Rationale:* review BLOCKING #8 — `repo` is a
  partition key and security boundary; an unresolved derivation can't underpin a
  filter. Rule fixed in § 5.2. *Alternative rejected:* leave it open — ships a
  security boundary on undefined data.
- **D4 — additive `serde(default)` parser widening.** *Rationale:* review MAJOR #4 —
  it's a parser change, not validated-layer-only; legacy files must parse.
  *Alternative rejected:* a schema-version bump forcing migration — heavy for an
  additive optional block.
- **D5 — subprocess git seam, no library.** Smallest dep; matches "doctrine builds
  the frame." *Alternative rejected:* `git2`/`gix`.
- **D6 — `verify` reuses the `adr status` `toml_edit` mutation shape.** No new
  mutation mechanism; authored-committed edit-preserving write.

## 8. Risks & Mitigations

- **R1 — `record` change breaks SL-005 record tests.** Intentional; update them this
  slice, keep `entity.rs` suites untouched. Bare `record` in a clean repo still
  succeeds (now anchored).
- **R2 — git portability/absence.** Every failure → `anchor_kind=none`/`None`, never
  panic; temp-repo + non-git-dir fixtures drive each edge.
- **R3 — repo-identity parsing variety (SSH/HTTPS/nested).** Normalize common forms,
  userinfo-stripped raw-URL fallback; unit table of remote-URL → identity.
- **R4 — secret capture via repo URL.** Strip userinfo before storing `repo`; test a
  credentialed URL maps to a clean identity.
- **R5 — legacy file parse regression.** `serde(default)` + an explicit
  SL-005-shaped fixture in the suite.
- **R6 — `verify` over-claims on a dirty tree** (HEAD ≠ working state). *Mitigation:*
  `verify` records `verified_sha = HEAD`; document that verification attests the
  committed state, not uncommitted edits (the reader's dirty-anchor handling is
  SL-008).

## 9. Quality Engineering & Validation

- **Git seam unit tests:** temp-repo fixture (init/commit/dirty/detached/unborn) →
  `head_frame` field assertions; non-git dir → `anchor_kind=none`; remote-URL → repo
  identity table (HTTPS, SSH, credentialed, no-origin, no-remote).
- **Parser tests:** SL-005-shaped legacy `memory.toml` parses (defaults); a
  fully-populated `[git]`/`[review]` round-trips through validation.
- **Verb integration:** `record --path … --command …` writes the scope arrays + a
  real `[git]` block with empty `verified_sha`; repo-scoped record in a non-git dir
  errors; `verify` stamps `verified_sha`/`reviewed`/`verification_state`
  edit-preservingly (comments survive); `show` prints the real anchor.
- **Behaviour-preservation:** entity/slice/state suites green unchanged; SL-005
  memory tests green except the intentionally-updated record/show assertions.
- **Gate:** `cargo clippy` zero warnings; `cargo fmt`; `just lint && just test` green
  per commit. (justfile lacks a `check` recipe — CLAUDE.md drift, tracked separately.)

## 10. Review Notes

> Revised from the adversarial review of the original combined SL-007 (codex,
> 2026-06-04): findings #2/#3/#4/#8 (the producer/anchoring blockers) are resolved
> here in a doc scoped to anchoring; the reader findings (#5 snapshot, #6 per-block
> nonce, #7 thread expiry) move to SL-008. Re-review this slice before `slice plan`,
> seeding the reviewer with: D1 (is `verify` the right minimal verb, or does
> attested staleness want a distinct `attest`/`reanchor` split?), D2/D3 (frame +
> repo-identity completeness against locked decision 6), and open Q1
> (`checkout_state_id` definition).
