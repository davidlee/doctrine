# Design SL-007: Memory anchoring & capture

> Revised after the pre-plan re-review ([audit.md](audit.md), 2026-06-04). The
> headline change: the born frame is **not invented here** — it is adopted from the
> interop counterparty (`forgettable`'s frozen `GitContextFrameV1`), reproduced
> byte-for-byte so doctrine records and backend claims dedup at the seam. Decisions
> Q-A…Q-E in the audit are folded in.

## 1. Design Problem

Build the **producer** that memory v1 retrieval depends on ([slice-007.md](slice-007.md)):
make `record` capture a memory's **scope** (where it lives) and a **git born
frame** (the commit it was true against), give the `Memory` parser the fields to
carry them, and add a minimal `verify` verb that advances the verification axis.
The reader (`find`/`retrieve`, ranking, staleness) is the sibling slice SL-008 and
is explicitly out of scope.

This is *not* a retrieval design. It is an **anchoring** design, and the three hard
parts are all on the write/IO side:

1. **The born frame is a locked, *shared* contract.** Locked decision 6
   ([memory-spec](../../../doc/memory-spec.md) § Locked decisions) fixes the minimum
   frame: `repo (+ repo_id_kind/confidence) + HEAD commit/tree/ref + checkout_state_id
   (dirty) + base_commit`. Critically, the **algorithm is frozen and shared with the
   event-store backend**: `forgettable` already defines this frame as
   `GitContextFrameV1` under the normalizer tags `forget.remote.v1` / `forget.checkout.v1`
   ([forgettable `src/git_context.rs`](../../../../forgettable/src/git_context.rs)).
   doctrine's first git seam (`src/git.rs`) **reproduces that algorithm byte-for-byte**
   — anything else derives divergent `repo_id`/`checkout_state_id` and breaks append
   idempotency when the adapter lands (interop constraint 3, § Identity). The only
   `git` token in `src/` today is `.git` as a root marker (`src/root.rs`).
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
   (no `[git]` data, no `reviewed`) still parse. Absent/empty `anchor_kind`
   **normalizes to `none` in validation** (serde defaults give `""`, not `"none"` —
   the normalization is explicit, not incidental).

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
  `repo=""`, `[git] anchor_kind="none"`, `[review] verification_state="unverified"`.
  It carries **none** of the keys the anchor/verification axes need (no `commit`,
  `tree`, `verified_sha`, `reviewed`, …) — this slice widens it.
- **Show framing.** `render_show` (`:595`) emits the data-block with `anchor: none`
  hardcoded and a comment naming SL-007 as the slice that fills it.
- **Mutation precedent.** `state::set_phase_status` (`src/state.rs`) and `adr status`
  (`src/adr.rs`) edit-preservingly mutate a toml via `toml_edit`. `adr status`
  targets an **authored committed** file and **refuses to insert a missing key**
  (the F-1 guard: a tail insert would land inside a trailing subtable) — the exact
  pattern `verify` reuses.
- **Counterparty reference.** `forgettable/src/git_context.rs` is the frozen
  `GitContextFrameV1` capture (repo identity, content-bearing `checkout_state_id`,
  normative flags, unstable-frame guards) doctrine reproduces. `hash.rs::sha256` and
  `canonical.rs::to_canonical_bytes` are the helper shapes doctrine mirrors.
- **No git module, no `verify`, no `clock`-style git seam.**

## 3. Forces & Constraints

- **Pure/imperative split (hard):** no git or clock in the pure layer. Frame capture
  is impure; `render_memory_toml` and validation take the resolved frame as data.
- **Locked decision 6 + interop byte-identity (hard):** the full born frame, and the
  *same bytes* forgettable derives — proven by a shared conformance golden-vector.
- **Interop constraint 4 (hard):** repo-scoped memory requires a born frame or it is
  an error; the backend never infers git — doctrine constructs the frame.
- **Storage rule:** scope + anchor are authored structured TOML (committed); `dirty`
  is **derived from `anchor_kind`, not stored**.
- **Behaviour-preservation:** `entity.rs` untouched, its suites green. `record`'s
  output changes intentionally; its tests update. Legacy `memory.toml` files must
  still parse (serde defaults + explicit empty→`none` normalization).
- **`verified_sha` semantics (hard, from review):** capture axis ≠ verification axis;
  `verify` refuses a dirty tree (it cannot honestly attest uncommitted state).
- **No git lib dep:** subprocess `git` only, under config-independent normative flags.

## 4. Guiding Principles

1. **Anchor honestly.** Write what capture knows (`base_commit`, `commit`/`checkout_state_id`,
   `tree`, `ref_name`); never write what only verification knows (`verified_sha`).
2. **Match the seam, don't invent it.** The frame algorithm is forgettable's; doctrine
   reproduces it and a golden-vector proves equivalence.
3. **Resolve git at the edge.** One frame capture per `record`; the pure render takes
   the frame as data.
4. **Every git failure is a state, not a crash.** Missing binary, non-repo, unborn →
   `anchor_kind = none`, surfaced explicitly; an unstable frame
   (submodule/symlink/multi-root) is a hard, named error, not a silent bad anchor.
5. **Widen additively.** New parser fields default-empty; absent `[git]`/`anchor_kind`
   normalizes to `none`; legacy files and the bare `record` flow keep working.
6. **Reuse the mutation seam.** `verify` is `adr status`-shaped `toml_edit` (refuse on
   missing key), writing atomically (temp+rename, § Concurrency).

## 5. Proposed Design

### 5.1 System Model

```
record ─▶ shell: clock::today()
                 git::capture(root) ─▶ Frame (or anchor_kind=none / hard error on unstable)
                 ─▶ Draft{ scope, frame, … } ─▶ render_memory_toml (pure) ─▶ materialise_named
verify ─▶ shell: git::capture(root) ─▶ refuse if dirty; else toml_edit mutate
                 [review].{verification_state,reviewed} + [git].verified_sha; atomic write
show   ─▶ Memory.anchor ─▶ render_show (pure, real anchor line)
```

`src/git.rs` (new, impure): `capture` (the full frame), repo-identity, content
fingerprints — reproducing `forgettable`'s `forget.remote.v1`/`forget.checkout.v1`.
`src/memory.rs`: parser widening, `Draft`/render widening, `verify` shell,
`render_show` anchor line.

### 5.2 Interfaces & Contracts

**Git seam (`src/git.rs`, impure).** Reproduces `forgettable`'s `GitContextFrameV1`
field-for-field; names align with the persisted schema.

```rust
pub(crate) enum AnchorKind { Commit, CheckoutState, None }
pub(crate) enum RepoIdKind { Explicit, Remote, LocalRoot }
pub(crate) enum Confidence { High, Medium, Low }

pub(crate) struct RepoIdentity {
  pub repo_id: String,            // normalized host[:port]/path, repo:git-root:<sha>, or ""
  pub kind: RepoIdKind,
  pub confidence: Confidence,
}

pub(crate) struct Frame {                     // the full locked-decision-6 frame
  pub anchor_kind: AnchorKind,
  pub repo: RepoIdentity,
  pub commit: String,                         // HEAD sha — set iff anchor_kind=Commit (clean)
  pub tree: String,                           // HEAD^{tree}
  pub ref_name: String,                       // refs/heads/...; "" detached (still anchored)
  pub checkout_state_id: String,              // set iff anchor_kind=CheckoutState (dirty)
  pub base_commit: String,                    // HEAD the memory sits on
}

pub(crate) enum CaptureError {                // unstable frame = hard, named error (Q-E)
  Unborn, NotARepo, MultiRoot, Submodule, Symlink, AmbiguousRemote, Git(String),
}

// Born clean/dirty/detached → Ok(Frame); unborn/non-repo → Ok(Frame{anchor_kind:None});
// submodule/symlink/multi-root/ambiguous-remote → Err (never an unstable anchor).
pub(crate) fn capture(repo_root: &Path) -> Result<Frame, CaptureError>;
```

Every git call runs under **normative flags** (`-c core.autocrlf=false -c core.eol=lf
-c core.fileMode=true`) so machine-local config cannot perturb the hash (parity with
forgettable; required for byte-identity).

- **HEAD / dirty.** `rev-parse --verify HEAD^{commit}` (born?), `rev-parse HEAD^{tree}`,
  `symbolic-ref --quiet HEAD` (empty ⇒ detached, still anchored). Dirty detection is
  **content-based**: `write-tree` index ≠ HEAD tree, or non-empty
  `diff HEAD --binary`, or untracked files present. `commit` is set **only** when
  clean (`anchor_kind=Commit`); when dirty (`anchor_kind=CheckoutState`) `commit` is
  empty and `base_commit` carries HEAD.
- **`checkout_state_id` (content-bearing, Q-C).** `sha256(canonical{ normalizer:
  "forget.checkout.v1", index_tree: git write-tree, worktree_fingerprint:
  sha256(git diff HEAD --binary --no-textconv --no-ext-diff), untracked_fingerprint:
  sha256(sorted untracked content-hashes) })`. Distinct edits to the same fileset do
  **not** collide.
- **Repo identity (`forget.remote.v1`).** Precedence explicit config →
  normalized remote → local-root fallback (`repo:git-root:<root_sha>`). Remote
  selection: preferred → `origin` → sole; **>1 remote without origin is
  `AmbiguousRemote`, not a guess.** `repo_id_kind`/`confidence` record which path was
  taken (remote/explicit = high; local-root = medium/low) — this is the
  partition/security boundary's trust signal. `normalize_remote_url` handles
  `ssh|https|http|git` URL forms and scp-short (`git@host:org/repo`), drops
  scheme/userinfo, preserves non-default ports, lowercases host, preserves path case.
  `--repo` overrides the derived `repo_id` (kind=`explicit`, confidence=high) — and is
  routed through the **same canonicalizer**, so a credentialed override is also
  userinfo-stripped.

**`record` flags (new).** `--path <P>` / `--glob <G>` / `--command <C>` (each
repeatable) → the `scope` arrays; `--repo <R>` overrides the derived identity.

**`verify` verb (new).**

```
doctrine memory verify <uid|key> [-p ROOT]
```

Resolves the memory (the `resolve_show` chokepoint), captures the frame, and:

- **Dirty tree ⇒ refuse** (Q-B): `verify` cannot honestly attest uncommitted state.
  Error tells the user to commit first.
- **Clean, born ⇒** `toml_edit`-mutates `[review].verification_state = "verified"`,
  `[review].reviewed = today`, `[git].verified_sha = <HEAD commit>`, bumps `updated`.
- **Non-git ⇒** stamps the review axis (`verification_state`/`reviewed`) but leaves
  `verified_sha` empty (honest: no SHA to attest; the time-based staleness mode uses
  this).

Edit-preserving and **atomic** (temp-then-`rename`, § Concurrency). Reuses the
`adr status` shape including the **F-1 missing-key guard** — `verify` refuses if the
keys it must edit (`[git].verified_sha`, `[review].{verification_state,reviewed}`) are
absent. Those keys are seeded at `record` (below), so a tool-authored memory always
has them.

**`Memory` widening.** Add `anchor: Anchor` (validated from `[git]`) and
`reviewed: String`; `Scope` gains `repo_id_kind`/`repo_id_confidence`. `Anchor`
mirrors `Frame`'s persisted subset + `verified_sha` + the `normalizer` tag.

### 5.3 Data, State & Ownership

- **Parser change.** `RawGit` gains `anchor_kind/commit/tree/ref_name/checkout_state_id/
  base_commit/verified_sha/normalizer`, all `#[serde(default)]`; **`dirty` is not a
  field** (derived from `anchor_kind`). `RawScope` gains `repo_id_kind/repo_id_confidence`
  (`#[serde(default)]`). `RawReview` gains `reviewed/review_by` (`#[serde(default)]`).
  Validation **normalizes** empty/absent `anchor_kind` → `None`, empty
  `repo_id_kind`/`confidence` → the lowest-trust default — explicitly, not via serde.
- **Template widening (B3).** `install/templates/memory.toml` gains placeholder keys
  for the full `[git]` block (`commit`/`tree`/`ref_name`/`checkout_state_id`/
  `base_commit`/`verified_sha=""`/`normalizer`), `[scope].repo_id_kind`/`confidence`,
  and `[review].reviewed=""`/`review_by=""`. `record` substitutes the captured frame +
  scope flags; `verified_sha`/`reviewed`/`review_by` render **empty** (capture writes
  neither axis) — present so `verify` can edit-preserve them under the F-1 guard.
  `render_memory_toml` widens to build the `[git]`/`[scope]` blocks from the `Frame` +
  flags (the `repo` field still lives in `[scope]`; the git facts in `[git]`).
- **Ownership.** `record` owns born-frame construction + scope capture; `verify` owns
  the verification axis; both write authored committed TOML. `dirty` and backlinks
  stay derived, never stored.

### 5.4 Lifecycle, Operations & Dynamics

- **record:** resolve root → `today` + `git::capture` → validate (**repo-scoped +
  `anchor_kind=none` ⇒ error**, constraint 4; unstable frame ⇒ the named
  `CaptureError`) → `Draft` → render → `materialise_named`. "Repo-scoped" predicate
  (m1): **a non-empty `repo` coordinate** (derived or `--repo`) requires a born frame;
  path/glob/command scopes alone do not.
- **verify:** resolve memory → `git::capture` → refuse if dirty → `toml_edit` mutate →
  atomic write. Idempotent in effect on a clean tree at the same HEAD (rewrites
  identical values + bumps `updated`/`reviewed`).
- **show:** unchanged flow; `render_show` now prints the real anchor.

### 5.5 Invariants, Assumptions & Edge Cases

- **Anchor honesty:** `verified_sha` is written by `verify` only, never by `record`;
  `verify` refuses a dirty tree.
- **Byte-identity:** doctrine's `repo_id`/`checkout_state_id` equal forgettable's for
  the same tree — pinned by a shared golden-vector (§ 9).
- **Repo-scoped + unanchorable ⇒ error**, not a silent unscoped write (constraint 4).
- **Legacy parse:** an SL-005 `memory.toml` (no `[git]` fields, no `reviewed`) parses;
  `anchor_kind` normalizes empty→`none`, `reviewed`→`""` — covered by a fixture.
- **Clean tree:** `anchor_kind=commit`, `commit=tree-HEAD`, `base_commit=HEAD`.
- **Dirty tree:** `anchor_kind=checkout_state`, `checkout_state_id` set, **`commit`
  empty**, `base_commit=HEAD`.
- **Detached HEAD:** `ref_name=""`, **still anchored** (`commit`/`checkout_state` by
  cleanliness); not `none`.
- **Unborn / non-repo:** `anchor_kind=none`; a repo-scoped record here errors.
- **Submodule / symlink / multi-root / ambiguous-remote:** hard named `CaptureError`,
  never an unstable anchor.
- **Secrets:** repo-identity strips URL userinfo (derived *and* `--repo`); no
  credential reaches `repo`.
- **No float anywhere** (n/a here — no scores; the frame is integer/string only).

## 6. Open Questions & Unknowns

*(All three pre-review questions resolved by the audit — recorded here for trail.)*

1. **`checkout_state_id` definition — RESOLVED (Q-C).** Content-bearing hash adopting
   forgettable's `forget.checkout.v1` (index tree + diff fingerprint + untracked
   fingerprint). No same-fileset collision.
2. **`verify` in a non-git repo — RESOLVED (Q-B).** Stamp the review axis with empty
   `verified_sha`; the time-based staleness mode uses it. (A *dirty* tree, by
   contrast, is refused.)
3. **Repo normalization corners — RESOLVED (Q-D).** Adopt forgettable's
   `normalize_remote_url` (SSH ports, scp-short, userinfo strip, multi-remote error).
   No remaining open question; future algorithm changes are caught by the normalizer
   tag + golden-vector.

## 7. Decisions, Rationale & Alternatives

- **D1 — `record` writes the born frame but NOT `verified_sha`; a minimal `verify`
  verb stamps it and refuses a dirty tree.** *Rationale:* review BLOCKING #2 —
  record-time `verified_sha` falsely attests every memory and kills the time-based
  mode. `verify` maps to the spec's `reviewed` event family. Refusing dirty (Q-B,
  review M4) keeps attestation honest — `verified_sha` never claims an uncommitted
  state. *Rejected:* stamp dirty + document (over-claims attested staleness);
  record-time `verified_sha` (spec deviation + dead-code).
- **D2 — reproduce forgettable's frame byte-for-byte (B4/B1).** *Rationale:* the
  event-store seam requires identical `repo_id`/`checkout_state_id` for append
  idempotency (interop constraint 3, § Identity). A hand-rolled frame bakes divergent
  ids into committed files — the wrong-contract-in-stored-data failure D2 itself warns
  of. The full frame (incl. `tree`, `repo_id_kind`/`confidence`) is the locked
  minimum. *Rejected:* invent doctrine's own frame (divergent at the seam); persist a
  subset (latent spec violation the reader inherits).
- **D3 — repo identity = forgettable's `forget.remote.v1` (B1/M3).** *Rationale:*
  `repo` is a partition key and security boundary; `repo_id_kind`/`confidence` are the
  trust signal, and the precedence + ambiguous-remote error make it deterministic. The
  naive `:`→`/` rule (original design) corrupts SSH-with-port and is non-deterministic
  across clones — refuted. *Rejected:* "first remote" fallback.
- **D4 — additive `serde(default)` widening + explicit empty→`none` normalization
  (M1).** *Rationale:* it's a parser change; legacy files must parse, and an absent
  `[git]` block must *mean* `none` (serde gives `""`, so validation normalizes).
  *Rejected:* a schema-version bump forcing migration.
- **D5 — subprocess git seam, no library.** Smallest dep; matches "doctrine builds the
  frame" and forgettable's own choice. *Rejected:* `git2`/`gix`.
- **D6 — `verify` reuses the `adr status` `toml_edit` shape, incl. the F-1 missing-key
  guard, with atomic temp+rename writes (B3/M6).** *Rationale:* no new mutation
  mechanism; the guard is safe only because `record` now **seeds** the verify-mutable
  keys (template widening). Atomic write satisfies § Concurrency. *Rejected:* tail
  `insert` of missing keys (corrupts the trailing subtable — the exact F-1 hazard);
  plain `fs::write` (not atomic). *(Note: `adr::set_adr_status` still uses plain
  `fs::write` — a pre-existing minor gap; a shared atomic-editor helper could later
  cover both. Out of scope to change adr here.)*
- **D7 — stay aligned with forgettable via re-implementation + a shared conformance
  golden-vector, not a shared crate (Q-D).** *Rationale:* forgettable is a separate
  workspace + daemon (PG/http); depending on its lib is too heavy. Re-implementing
  pinned to the normalizer tags keeps doctrine independent; the golden-vector catches
  drift. *Rejected:* extract a shared crate now (cross-repo restructure — a future
  slice/ADR if drift proves real).
- **D8 — adopt forgettable's unstable-frame guards (Q-E).** Born/unborn/non-repo are
  three states; submodule/symlink/multi-root/ambiguous-remote are hard errors.
  *Rationale:* never emit an unstable anchor; parity with the seam. *Rejected:*
  best-effort anchoring with a documented gap (a divergence to reconcile later).

## 8. Risks & Mitigations

- **R1 — `record` change breaks SL-005 record tests.** Intentional; update them this
  slice, keep `entity.rs` suites untouched. Bare `record` in a clean repo still
  succeeds (now anchored).
- **R2 — git portability/absence.** Every soft failure → `anchor_kind=none`/named
  error, never panic; temp-repo + non-repo fixtures drive each edge.
- **R3 — frame drift from forgettable (the seam breaks silently).** *Mitigation:* the
  shared conformance golden-vector (§ 9) fails CI if `repo_id`/`checkout_state_id`
  diverge; the persisted `normalizer` tag versions the algorithm.
- **R4 — secret capture via repo URL.** Strip userinfo before storing `repo` (derived
  *and* `--repo`); test a credentialed URL maps to a clean identity.
- **R5 — legacy file parse regression.** `serde(default)` + explicit empty→`none`
  normalization + an SL-005-shaped fixture.
- **R6 — `verify` over-claims on a dirty tree.** *Resolved* (D1/Q-B): `verify`
  refuses a dirty tree; it never stamps `verified_sha` against uncommitted state.
- **R7 — unstable frame anchored silently.** *Mitigation* (D8): submodule/symlink/
  multi-root/ambiguous-remote are hard named errors with fixtures.

## 9. Quality Engineering & Validation

- **Git seam unit tests:** temp-repo fixture (clean/dirty/detached/unborn/non-repo) →
  `capture` field assertions (commit empty when dirty, detached still anchored, none
  on unborn); remote-URL → repo-identity table (HTTPS, SSH, SSH-with-port, scp-short,
  credentialed, no-origin→ambiguous-error, no-remote→local-root); unstable trees
  (submodule/symlink/multi-root) → the named errors.
- **Conformance golden-vector (D7):** a fixture tree with known `repo_id` +
  `checkout_state_id` values, asserted equal to forgettable's reference output —
  the byte-identity proof for the interop seam.
- **Parser tests:** SL-005-shaped legacy `memory.toml` parses (defaults + empty→none);
  a fully-populated `[git]`/`[review]` round-trips through validation.
- **Verb integration:** `record --path … --command …` writes the scope arrays + a
  real `[git]` block with empty `verified_sha` + seeded `reviewed`/`review_by`;
  repo-scoped record in a non-git dir errors; `verify` on a clean tree stamps
  `verified_sha`/`reviewed`/`verification_state` edit-preservingly (comments survive,
  write is atomic); `verify` on a dirty tree refuses; `show` prints the real anchor.
- **Behaviour-preservation:** entity/slice/state suites green unchanged; SL-005
  memory tests green except the intentionally-updated record/show assertions.
- **Gate:** `cargo clippy` zero warnings; `cargo fmt`; `just lint && just test` green
  per commit. (justfile lacks a `check` recipe — CLAUDE.md drift, tracked separately.)

## 10. Review Notes

> Re-reviewed before `slice plan` (two independent adversarial passes — this agent +
> codex; [audit.md](audit.md), 2026-06-04). The pass found the design hand-rolled a
> frame the interop counterparty (`forgettable`) had already frozen; the resolution
> (D2/D7/D8) is to reproduce it byte-for-byte with a conformance vector. Decisions
> Q-A (extend the locked `[git]`/`[scope]` schema — done in memory-spec), Q-B (refuse
> dirty `verify`), Q-C (content-bearing `checkout_state_id`), Q-D (re-implement +
> vectors), Q-E (adopt the unstable-frame guards) are folded in, along with the
> design-only fixes B2 (commit empty when dirty), B3 (template seeds verify-mutable
> keys), M1 (empty→none normalization), M2 (detached still anchored), M6 (atomic
> write), m1 (repo-scoped = non-empty `repo`). Design holds — ready for `slice plan 7`.
