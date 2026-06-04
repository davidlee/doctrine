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
