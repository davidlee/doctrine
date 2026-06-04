# Audit — SL-007 memory-anchoring · design re-review (pre-plan gate)

> Hand-authored (no `slice audit` scaffold — CLAUDE.md known gap). Adversarial
> re-review of `design.md` @ `47a6b99`, before `slice plan`. Two independent
> passes: this agent + codex (read-only, REFUTE-prompted) — they converged.
> Reviewers seeded with handover §"Review scope". **Outcome: design does NOT yet
> hold — 3 BLOCKING must resolve before plan.**

## Verdict

Re-review **fails**. The producer shape is sound; the born-frame contract is
not. The headline (B1) needs a LOCKED-spec reconciliation — a user decision, not
an agent edit. B2/B3 are design-only fixes. Resolve BLOCKING + decide M4/M5, then
re-gate.

## Confirmed clean (the original review's producer blockers)

- **#2 (verified_sha at capture) — CLOSED.** `record` writes no `verified_sha`;
  template (`install/templates/memory.toml:18`) + `render_memory_toml`
  (`src/memory.rs:460`) have no field/substitution for it. Capture axis ≠
  verification axis holds. ✅
- **D1 verb-count — OK.** A single `verify` maps cleanly to the spec's `reviewed`
  event family (`memory-spec.md` § Lifecycle ledger:246); conflating the portable
  `review`/`verify` ops into one verb is acceptable for v1. The real D1 defect is
  the dirty-tree behaviour (M4), not the verb count.

## BLOCKING

- **B1 — "full locked-decision-6 frame" is false in BOTH directions.**
  `design.md` §5.2/§5.3 *drops* the mandated `repo (+ id kind/confidence)`
  (`memory-spec.md` § Scope & anchoring:290-291 lists it in the *minimum* frame)
  and *adds* persisted `tree`/`dirty` that the spec `[git]` schema
  (`memory-spec.md:157-164`) does not define. The stored contract is internally
  inconsistent with the umbrella, and `dirty` is anyway derivable from
  `anchor_kind` (derive-don't-store). **Fix:** reconcile the LOCKED spec first
  (one canonical frame/schema pair), then make the design persist exactly that.
  *Decision needed* — see Open decisions Q-A. Without `id kind/confidence` the
  repo partition/security boundary (D3's whole rationale) cannot tell a
  high-confidence `origin` match from a guessed first-remote.

- **B2 — `commit` written on a dirty tree violates the schema.** Spec:
  `commit` is "set iff `anchor_kind = commit` (clean tree)" (`memory-spec.md:159`).
  Design §5.5 keeps `commit = dirty-base HEAD` under `anchor_kind=checkout_state`.
  **Fix:** on `checkout_state`, `commit = ""`; carry the HEAD in `base_commit`,
  the state in `checkout_state_id`. Design-only.

- **B3 — the `verify` write path is self-contradictory.** Design reuses the
  `adr status` `toml_edit` shape (D6), whose F-1 guard *refuses* missing keys
  (`src/adr.rs:196-200`). But the template seeds neither `[git].verified_sha` nor
  `[review].reviewed`/`review_by` (`install/templates/memory.toml:18-22`), so
  `verify` on a freshly-recorded memory would refuse. §5.3's "template keeps its
  keys" is wrong — those keys don't exist. **Fix:** seed all verify-mutable keys
  at `record`/template render (`verified_sha=""`, `reviewed=""`, `review_by=""`),
  OR drop the F-1 reuse claim and define a safe insert-into-existing-table
  strategy. Design-only; prefer seeding (keeps the adr pattern intact).

## MAJOR

- **M1 — legacy-parse claim overstated.** §5.5 says an absent `[git]` block
  "parses to `anchor_kind=none` via serde defaults" — but `#[serde(default)]` on a
  `String` yields `""`, not `"none"`. **Fix:** define + test an explicit
  empty/absent→`None` normalization in `Anchor` validation; don't attribute it to
  serde defaults. (Files still *parse*; the *value* is wrong without normalization.)

- **M2 — detached HEAD misclassified as unanchorable.** §5.5 collapses detached
  HEAD to `anchor_kind=none`, discarding a real commit/tree/base_commit and
  spuriously tripping the repo-scoped error gate. Spec treats detached HEAD as a
  *reader-side* reachability question (`memory-spec.md:335`), not an absent anchor.
  **Fix:** detached HEAD is anchored (`commit`/`checkout_state` by cleanliness)
  with `ref_name=""`; reserve `none` for unborn/non-git only.

- **M3 — repo identity not stable enough for a partition/security boundary.**
  (a) "first remote" is clone-order/naming dependent → unstable across clones;
  (b) `:`→`/` corrupts SSH URLs with `:port` (`ssh://git@h:22/o/r`); (c) `--repo`
  override bypasses the userinfo-strip/normalization. **Fix:** one deterministic
  canonicalization pipeline for *both* derived and `--repo` values (SSH-with-port
  handled), and a refusal/explicit-low-confidence path for ambiguous multi-remote.
  Ties to B1's `id kind/confidence`.

- **M4 — `verify` over-claims on a dirty tree (R6, accepted-as-documented).**
  Stamping `verified_sha = HEAD` while the tree is dirty silently promotes the
  memory into the spec's "scoped + attested" staleness mode
  (`memory-spec.md:331`) though the verification may have been against uncommitted
  state. *Decision needed* — see Q-B.

- **M5 — `checkout_state_id = hash(porcelain + HEAD)` too weak for an id.** Two
  distinct content edits over the same path/status set collide; untracked files
  perturb it. *Decision needed* — see Q-C.

- **M6 — `verify` "atomic write" claim unmet.** §5.4 promises atomic; the reused
  `adr::set_adr_status` is a plain `fs::write` over the original path
  (`src/adr.rs:203`), not the spec's temp-then-rename (`memory-spec.md` §
  Concurrency:454). **Fix:** either route `verify` through a shared atomic editor
  (read→toml_edit→temp→rename) or drop the "atomic" wording and accept
  single-writer disposability. (Codex rated this BLOCKING; downgraded — no daemon,
  no concurrent writer — but it is a real spec/word mismatch to settle.)

## MINOR

- **m1 — "repo-scoped" predicate undefined.** The born-frame-required error gate
  (§5.4/§5.5, constraint 4) never says whether "repo-scoped" means non-empty
  `scope.repo`, any path/glob/command scope, or a combination. **Fix:** pin one
  exact predicate in the design + a test; enforce at `record` pre-render.
- **m2 — `verify` never sets `review_by` (the horizon).** The `reviewed` event is
  "verification + horizon" (`memory-spec.md:246`); v1 leaves the horizon empty.
  Acceptable omission — note it explicitly as deferred, don't leave it silent.

## Open decisions (need the user — block folding into design.md)

- **Q-A (from B1, touches the LOCKED spec):** reconcile the frame/schema. Options:
  (1) extend the spec `[git]` schema to include `tree` + `repo_id_kind`/
  `repo_id_confidence`, drop persisted `dirty` (derive from `anchor_kind`);
  (2) amend locked decision 6's frame wording to match a leaner persisted shape.
  The spec is LOCKED — this is a spec edit, not an agent call.
- **Q-B (from M4):** dirty-tree `verify` — refuse / stamp a distinct
  dirty-verified state / accept + document. Affects whether attested staleness can
  be trusted.
- **Q-C (from M5):** `checkout_state_id` definition — accept the cheap colliding
  hash for v1 (documented), or define a content-bearing id now, or refuse dirty
  anchoring until specified.

## Round 2 — the interop counterparty already exists (`forgettable`)

`/workspace/forgettable/src/git_context.rs` is the event-store backend's **frozen
reference implementation of the same born-frame** doctrine must produce. memory-spec
§ Backend abstraction binds doctrine's memory stream to forgettable's generic event
store; § Identity:278 requires a doctrine `record` and a backend claim to **dedup at
the seam** — which needs byte-identical `repo_id` and `checkout_state_id`. The
design's hand-rolled rules diverge from forgettable's frozen normalizers
(`forget.remote.v1`, `forget.checkout.v1`) → divergent ids baked into committed
memory files → append idempotency (interop constraint 3) breaks when the adapter
lands. This elevates to:

- **B4 (BLOCKING) — the frame algorithm must MATCH forgettable, not be invented.**
  doctrine's `src/git.rs` must reproduce `GitContextFrameV1` semantics field-for-field
  (the same canonical-bytes rule, the same normalizer tags), or stored anchors are
  un-exportable with stable ids. **Fix:** adopt the forgettable algorithm (decision
  Q-D below); add a shared conformance golden-vector so CI proves the two agree.

**Findings RESOLVED by adopting forgettable's reference** (it is the airtight version
the seeds demanded):

- **B1 / M3 (repo identity):** `RepoIdentity { repo_id, repo_id_kind ∈
  {explicit,remote,local_root}, confidence ∈ {high,medium,low}, remote_url_raw/
  normalized, root_commit, alternate_remotes, … }`. Precedence explicit→remote→
  local-root; **errors `AmbiguousRemote`** on >1 remote without origin/preferred (not
  "first remote"); fork warning on origin+upstream. `normalize_remote_url` handles
  SSH `:port` (scheme default-port drop), scp-short, userinfo-strip, host-lowercase/
  path-case — with a full test table. This *is* the "id kind/confidence" the spec
  frame mandates and the stable boundary D3 needs.
- **B2 (commit on dirty):** forgettable's `Anchor` is an enum — `Commit{commit}` XOR
  `Checkout{checkout_state_id}`. Dirty carries no `commit`. Confirms the fix.
- **M2 (detached HEAD):** `HeadAnchor { ref_name: Option, detached: bool }` — detached
  is still anchored (commit/checkout), `none` reserved for unborn/non-repo. Confirms.
- **M5 / Q-C (checkout_state_id):** `checkout_state_id = sha256(canonical{normalizer,
  index_tree = git write-tree, worktree_fingerprint = sha256(git diff HEAD --binary),
  untracked_fingerprint = sha256(sorted untracked content-hashes)})`. Content-bearing —
  does NOT collide on same-fileset different-content edits. This is exactly Q-C
  option 2, already implemented + tested.

**New must-adopt (forgettable does, design omits):**

- **NORMATIVE_FLAGS** on every git call (`core.autocrlf=false`, `core.eol=lf`,
  `core.fileMode=true`) so machine-local config can't perturb the hash. Design omits —
  without it the frame is not reproducible across machines.
- **Born/unborn + non-repo are three states**, not two; submodule (160000) / symlink
  (120000) / multi-root rejected rather than emitting an unstable frame (m12/DEC-002-06).
  Decide whether v1 adopts these guards or defers (defer = a known unstable-frame gap).
- **Carry the normalizer tags** (`forget.remote.v1`/`forget.checkout.v1`) in the
  persisted anchor so a future algorithm change is detectable at the seam.

## Decisions (this review)

- **Q-A → Extend the spec schema.** Add `tree` + `repo_id_kind`/`repo_id_confidence`
  (mirror forgettable's `RepoIdentity` field names) to the LOCKED `[git]`/`[scope]`
  schema; drop persisted `dirty` (derive from `anchor_kind`). Spec edit required
  before the design can claim conformance.
- **Q-B → Refuse `verify` on a dirty tree.** No `verified_sha` stamped against
  uncommitted state; verify after committing. (Born-dirty `record` anchors stay
  first-class — you just can't *attest* a dirty tree.)
- **Q-C → Content-bearing `checkout_state_id`**, adopting forgettable's exact
  composition (resolves M5).
- **Q-D → Re-implement in `src/git.rs`** pinned to the same normalizer tags
  (`forget.remote.v1`/`forget.checkout.v1`) + a shared conformance golden-vector in
  CI proving doctrine and forgettable derive identical `repo_id`/`checkout_state_id`.
  doctrine stays independent of forgettable's daemon; a shared crate is a future
  consolidation (its own slice/ADR) if drift proves real.
- **Q-E → Adopt forgettable's unstable-frame guards.** Reject submodule (160000) /
  symlink (120000) / multi-root; split born/unborn/non-repo as three states. Never
  emit an unstable frame.

All decisions resolved — design.md folding + the Q-A spec edit can proceed.

## Disposition

- BLOCKING B2, B3 + MAJOR M1, M2, M3, m1, m2: design-only — fold into `design.md`
  once Q-A/B/C land (some interact with the reconciliation).
- BLOCKING B1 + decisions Q-A/B/C: surfaced to the user. **No design or spec edits
  made by the reviewer** (correctness-first; LOCKED spec).
- Gate held: no plan scaffolded, no code. `slice plan 7` only after the design
  holds.

## Close-out (PHASE-06 — slice complete)

All BLOCKING/MAJOR/MINOR findings landed across PHASE-01..06 and are green:

- **B1/M3, B4, Q-A/D/E** — `src/git.rs` reproduces forgettable's
  `forget.remote.v1`/`forget.checkout.v1` byte-for-byte; the conformance
  golden-vector + verbatim normalize-url table pin byte-identity (PHASE-01/02).
- **B2** — dirty → `anchor_kind=checkout_state`, `commit=""`, HEAD in `base_commit`
  (PHASE-02, `record_in_a_dirty_repo`).
- **B3/D6, M6** — `verify` reuses the adr `toml_edit` F-1 guard over
  template-seeded keys, written via `fsutil::write_atomic` (temp+rename) (PHASE-04/05).
- **M1** — explicit empty/absent→`None` normalization in `Anchor`/`Scope`
  validation, with the legacy-compat fixture (PHASE-03).
- **M2** — detached HEAD stays anchored (`ref_name=""`), `none` reserved for
  unborn/non-repo (PHASE-02); `show` renders `ref detached`.
- **m1** — repo-scoped predicate = non-empty `repo_id` (derived or `--repo`), the
  constraint-4 gate (PHASE-04).
- **Q-B/M4** — `verify` refuses a dirty tree; non-git stamps the review axis with
  empty `verified_sha` (PHASE-05).
- **m2** — `review_by` (the verification horizon) is parsed/carried but left empty
  by `verify` in v1 (documented deferral, F6).

Two committed-output judgement calls (notes F7) **confirmed**, see notes **F9**:
the flat `[git].normalizer` placement and the unborn/non-git constraint-4
asymmetry both hold for v1 — `show` reads anchor presence only.

**Gate:** 214 lib tests + 1 e2e (`tests/e2e_memory_anchoring.rs`, the
record→commit→verify→show→list loop over the built binary) green; `cargo clippy`
zero, `cargo fmt` clean; entity/slice/state suites unchanged (behaviour-preserved).
SL-007 producer surface complete. Reader (`find`/`retrieve`, ranking, staleness)
is SL-008.

## /code-review (close-out · 2026-06-05)

> Adversarial review of the SL-007 producer diff `712e433..51809e3` (~2k lines)
> against `design.md` + `memory-spec.md` + the frozen counterparty
> `forgettable/src/git_context.rs`. Each finding verify-or-refute'd against design
> intent before recording. Two **new reproducible defects** the prior passes (A-1
> stored-escaping, A-2 body-nonce) missed; the byte-identity claim re-verified at
> the source level; carried decisions re-tested and held.

### Byte-identity re-verification (the highest-risk lens, D2/D7)

Performed a **source-level field-for-field diff** of doctrine's ported derivations
against forgettable's frozen originals (the strongest proof short of running
forgettable, which F4 admits was not done):

- `git::canonical_bytes` ≡ `canonical::to_canonical_bytes` — byte-identical
  (key-sort, minimal escaping, integer-only/float-reject, `\u00XX` control path).
- `git::sha256` ≡ `hash::sha256` — both `hex::encode(Sha256)`, lowercase.
- `git::checkout_state_id` ≡ `git_context::checkout_state_id` — identical `json!`
  composition + `sha256(canonical)`.
- `git::normalize_remote_url` + `scheme_info`/`clean_path`/`host_and_port` —
  verbatim copy.

**Verdict:** the `repo_id`/`checkout_state_id` algorithm — the only bytes the
interop seam dedups on (spec § Identity) — is genuinely byte-identical. The
self-anchored golden vector (F4) is therefore an adequate *regression* pin, not a
*conformance* proof; current divergence risk is nil, future-drift risk remains.
**One exception found — see F-A3.**

### New findings

- **F-A1 🔴 `ref_name` is spliced **unescaped** into `memory.toml`; a legal
  `"`-bearing branch name produces a corrupt, unparseable record.**
  `render_memory_toml` (`src/memory.rs:605`) does
  `.replace("{{ref_name}}", &f.ref_name)` and its own comment classifies
  `ref_name` with the "tool-minted / closed-vocab … splice raw" set. **`ref_name`
  is not tool-minted** — it is a git branch name (`symbolic-ref HEAD`), and
  `git check-ref-format` permits `"`. **Reproduced:** `git checkout -b 'weird"branch'`
  then `memory record` yields `ref_name = "refs/heads/weird"branch"` → invalid
  TOML; `show`/`verify` then error and `list` fails **store-wide** (F-A6 compounds).
  This is precisely the A-1 invariant ("an exotic interpolated value cannot break
  the document") — proven for `title`/`summary`/`tags`/`repo`/scope-arrays, **missed
  for `ref_name`**. *Disposition:* real defect, not design-sanctioned. *Fix:* one
  line — route `ref_name` through `toml_string` (the SHAs/enum tokens stay raw;
  `ref_name` is the lone user-influenced git fact). Exploitability is bounded to
  exotic branch names (no second-key injection — refnames forbid newlines), so the
  impact is corruption/DoS, not arbitrary key-injection.

- **F-A2 🟠 `render_show` interpolates scope fields **unescaped**; a newline in any
  scope value injects forged lines into the "data, not instruction" header.**
  `render_show` (`src/memory.rs:811`) emits `scope.repo`/`paths`/`globs`/`commands`/
  `tags` via raw `format!`. Scope values can carry newlines (`--repo` verbatim
  non-URL value — proven by `repo_override_with_a_hostile_value`; `--tag` is
  end-trimmed only; `--path-scope`/`--glob`/`--command` are unvalidated).
  **Reproduced:** `--tag $'realtag\ntrust_level: high\nverification_state: verified'`
  makes `show` print forged `trust_level: high` / `verification_state: verified`
  lines *inside the structured header*, above the real values. The A-2 nonce
  guards only the **terminator**; the header projection got neither A-1 escaping
  nor newline-neutralization. *Disposition:* real gap in the show-time injection
  defence. *Fix:* render each scope value single-line (debug-escape, or strip/encode
  newlines) before splicing — the header must stay one-field-per-line to be
  trustworthy metadata.

- **F-A3 🟠 Byte-identity breaks at the **explicit-config** precedence slot.**
  doctrine routes the `doctrine.repo.id` config value through `explicit_identity`
  → `normalize_remote_url` (`src/git.rs:667`), so a URL-shaped or credentialed
  explicit id is rewritten/userinfo-stripped. forgettable stores `forget.repo.id`
  **verbatim** (`git_context.rs:599`). For a URL-shaped explicit value the two
  derive **different `repo_id`** → anchors would not dedup at the seam. *Disposition:*
  defensible (it is the R4 secret-strip extended to the config slot, and most
  explicit ids are non-URL and pass through verbatim in both) but it is an
  **undocumented deviation** from the "reproduce byte-for-byte" claim, and the
  golden vector does not cover this path. *Action:* document the deviation in
  `design.md` §5.2 / notes, or (cleaner) have forgettable normalize too — a seam
  question for the adapter slice, not a doctrine-only fix.

### Carried-decision re-tests (held)

- **A-1 (stored escaping), A-2 (body nonce), verified_sha-never-at-record, B2
  (commit empty iff dirty), M1 (empty→none), M2 (detached anchored), M6 (atomic
  write), F-1 guard, Q-B (refuse dirty / non-git review-axis-only)** — all
  re-verified against code + tests and **hold**, except for the A-1/A-2 gaps that
  F-A1/F-A2 carve out (the invariants are right; their *coverage* is incomplete).
- **F7/F9 judgement calls** (flat `[git].normalizer` iff checkout_state; unborn
  repo-scoped errors while non-git succeeds unscoped) — re-confirmed; `show` reads
  anchor presence only.

### Lower-severity observations

- **F-A4 🟡 `forget.repo.preferred_remote` (forgettable) is itself a malformed git
  config key** (underscore in the variable segment — git rejects it; the same
  reason F5 renamed doctrine's to `preferredremote`). forgettable therefore can
  never read its preferred-remote via standard `git config`; doctrine's *works*.
  The two thus select remotes differently when a preferred remote is configured →
  a latent `repo_id` divergence on the remote slot. Interop seam note (adapter
  slice / a forgettable bug, not doctrine's).

- **F-A5 🟡 The module-wide `#![cfg_attr(not(test), expect(dead_code, …))]` on
  `memory.rs` is now over-broad.** All consumers (record/show/verify/list) are
  wired, yet the blanket suppression — and its stale "consumers wired by … PHASE-05"
  reason — remains, masking genuinely carried-but-unread fields (`Anchor.tree`/
  `base_commit`/`normalizer`, `Memory.severity`/`weight`/`review_by`/`reviewed`).
  Newly-dead code will not be caught. *Action:* narrow to the specific unread items
  (or `_`-prefix / `#[allow]` with reasons) so the lint regains its teeth — a
  "confidence to change" cleanup, not a correctness bug.

- **F-A6 🔵 One malformed `memory.toml` fails `list` entirely.** `collect_memories`
  propagates the first parse error (design-sanctioned: tool-authored store). But it
  **compounds F-A1**: a single corrupt record (e.g. from the `ref_name` bug) makes
  the whole store unlistable. Worth a `list`-resilience reconsideration in SL-008
  (skip+warn the bad row rather than abort).

- **F-A7 🔵 `verify` attests the cwd HEAD with no check that the memory's stored
  `repo`/anchor matches the captured repo.** Running `verify` from an unrelated
  repo stamps a meaningless `verified_sha`. Single-repo workflow makes this benign
  in v1; the staleness reader (SL-008) should gate on repo-id match.

- **F-A8 🔵 `git_opt` collapses every non-zero exit to `None`.** A transient or
  corrupt-repo failure on `rev-parse --is-inside-work-tree` / `HEAD^{commit}`
  masquerades as non-repo / unborn → a silent unscoped write instead of an error.
  Parity with forgettable; acceptable for v1, flagged for awareness.

- **F-A9 🔵 The single e2e covers only the happy path** (clean record→verify→show).
  Dirty-refuse, non-git, and constraint-4 error paths are unit-only — never
  exercised over the real binary.

### Disposition

The producer is sound and the interop algorithm is genuinely byte-identical.
**F-A1 (🔴)** is a real correctness defect with a one-line fix and is worth landing
on the slice before it is truly closed; **F-A2 (🟠)** is a real show-time injection
gap (one-line-render fix). **F-A3 (🟠)** and **F-A4 (🟡)** are interop-seam
documentation/adapter concerns. **F-A5 (🟡)** is a maintainability cleanup. The
🔵 items are SL-008 / awareness notes. No behaviour-preservation breach found
(entity/slice/state engine untouched).

**Update 2026-06-05:** F-A1 + F-A2 fixed (`8b19370`, `fix(SL-007): escape
ref_name + scrub show header`). Regression tests added; `just check` green
(216 lib + 1 e2e). F-A3/A4/A5 and the 🔵 items remain as recorded.

## /code-review (close-out · 2026-06-05 · round 2 — `list`/`show` ergonomics)

> Surfaced by real use (`doctrine memory list` then `show` a listed id). Three
> read-path defects in the SL-005 surface widened by SL-007. The first is the
> same unescaped-render class as F-A2 but in `format_list`; the other two make
> `list` output non-actionable.

- **F-A10 🟠 `format_list` emits `m.title` unescaped — a newline in a title
  breaks the row (and can forge a fake row).** Same class as F-A2 (which fixed
  `render_show`), missed in `format_list` (`src/memory.rs`). Observed live: a
  title `"this is a test memory\npew pew i'm remembenening"` renders as two
  lines under one uid. *Fix:* `scrub_line` the title in `format_list` (the helper
  added for F-A2). Titles should also arguably be newline-rejected at `record`,
  but scrubbing the render is the defensive fix that also covers legacy rows.

- **F-A11 🟠 `list`'s id column is non-actionable: it prints `short_uid` (12
  chars = `mem_` + 8 hex), but `show`/`MemoryRef::parse` require the full
  `mem_` + 32-hex uid.** Copy-pasting a listed id into `show` always errors
  (`not a valid memory uid or key`). *Fix:* print the full uid in `list` (widen
  the column) so the output drives `show`.

- **F-A12 🟠 `short_uid` collides for uuid-v7 ids.** uids are time-ordered
  (uuid-v7: leading bytes = ms timestamp), so the first 8 hex are a coarse
  timestamp bucket shared by memories recorded close together. Observed live:
  two distinct memories both rendered `mem_019e922c`. So a short-id column is
  inherently ambiguous, and "let `show` accept the short form" is unsafe without
  unique-prefix resolution. *Fix (optional, ergonomics):* teach `show`/
  `MemoryRef` to resolve a **unique uid prefix** (scan `items/`, require ≥N hex,
  error on ambiguity) — needs a collision check, more surface than F-A10/A11.

*Disposition:* F-A10 (🟠, injection class) + F-A11 (🟠, broken affordance) are
small render/format fixes worth landing; F-A12 (🟠) is the ergonomic follow-on
(prefix resolution) and touches the resolver. **Handed to a fresh agent** for
implementation (all three) — see `handover.md`.
