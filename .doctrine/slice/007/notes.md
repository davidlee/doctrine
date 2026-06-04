# Notes SL-007: Memory anchoring & capture: record scope+git frame, verify, git seam

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Findings (durable)

### F1 — `canonical_bytes` returns `Result`, not `Vec<u8>` (PHASE-01)

plan.toml EX-2 prose says `canonical_bytes(&Value)->Vec<u8>` *and* "float-rejecting".
Incompatible: rejection under `panic="deny"`/`unwrap_used="deny"` can't be a bare
`Vec<u8>` return. Implemented as `Result<Vec<u8>, NonIntegerNumber>`, matching
forgettable's `to_canonical_bytes`. Internal callers (`checkout_state_id`) use
`.unwrap_or_default()` on all-string payloads that never error. **No further
action** — correctness-driven deviation from loose plan prose; criteria intent met.

### F2 — `CaptureError::Unborn` design/plan conflict — RESOLVE IN PHASE-02

`design.md` §5.2 lists `Unborn` as a `CaptureError` variant, but §5.5 + plan.toml
PHASE-02 EX-2/VT-1 say **unborn/non-repo → `Ok(Frame{anchor_kind:None})`** (a
captured None-anchor state, not an error). forgettable agrees: unborn → partial
non-writable frame (`Ok`); only non-repo is its own error there.

Resolution for PHASE-02: unborn **and** non-repo are the `AnchorKind::None` *Ok*
states (design §5.5 + constraint 4 — a repo-scoped record over a None frame is what
errors, at the `record` layer in PHASE-04). `CaptureError` carries only the
*unstable-frame* guards + git failures: `MultiRoot, Submodule, Symlink,
AmbiguousRemote, Git(String)`. Drop `Unborn` as an error variant. **Open
sub-question:** non-repo — `Ok(None)` (design §5.5) vs forgettable's
`NotARepository` error. design §5.5 wins for doctrine (None state, so a non-git
`record` can still write an unscoped memory; `verify` in non-git stamps the review
axis per Q-B). Confirm against §5.4 record flow before coding.

**RESOLVED in PHASE-02.** `CaptureError = { MultiRoot(usize), Submodule, Symlink,
AmbiguousRemote(Vec<String>), Git(String) }` — both `Unborn` *and* `NotARepo`
dropped as error variants. capture returns `Ok(none_frame())` for non-repo (empty
`repo_id`, `LocalRoot`/`Low`) and `Ok(Frame{anchor_kind:None})` for unborn (a repo,
so identity is still derived — a remote-only unborn repo gets a `Remote`/`High` id).
Spawn/UTF-8/non-zero-exit fold into `Git(String)`. The §5.4 record-flow scope gate
(constraint 4: repo-scoped + None → error) lands PHASE-04 at the `record` layer, not
here. No design.md change needed — §5.2's enum list was the loose superset; §5.5/
EX-2 are authoritative and now matched.

### F3 — `capture(repo_root)` signature stayed locked; `--repo` is a sibling fn (PHASE-02)

The `--repo` override (record's flag, PHASE-04) does **not** thread through
`capture`. Instead `explicit_identity(raw) -> RepoIdentity` (pub(crate)) builds the
`Explicit`/`High` id, routed through `normalize_remote_url` so a credentialed value
is userinfo-stripped (R4) and a non-URL value (`org/project`) is kept verbatim.
capture itself reads git config `doctrine.repo.id` for the config-explicit
precedence slot, reusing the same fn. PHASE-04 overrides the captured `frame.repo`
via `explicit_identity` when `--repo` is passed. Keeps capture's contract clean and
the explicit path unit-testable now (VT-2).

### F4 — golden-vector literal anchored from doctrine's capture, untracked-only (PHASE-02)

`conformance_golden_vector` pins `repo_id="github.com/org/repo"` +
`checkout_state_id=88d9489028e302700c2e6430e6df1d06539dccfd283d2ed99995258482ccf86c`.
The fixture is **untracked-only dirty** by design: every csid input is then a
git-frozen object hash — `index_tree`=HEAD tree SHA, `worktree_fingerprint`=
sha256(empty `diff HEAD`), `untracked_fingerprint`=sha256(path + blob SHA). None
depend on commit dates or git version, so the literal is reproducible. Cross-impl
equality with forgettable rests on the byte-copied frozen fns (VT-1 verbatim table +
the canonical/sha256/csid helper tests), **not** on a freshly-run forgettable —
forgettable was not built/run this phase (daemon+PG workspace; D7 = mirror, not
depend). If a stronger proof is wanted, run forgettable's `capture` on the same
fixture and diff. Low risk: drift in either impl breaks the literal.

### F5 — git config keys are alphanumeric/`-` only, no `_` (PHASE-02)

`doctrine.repo.preferred_remote` is an **invalid** git config key (`git config …`
errors `invalid key`). The const is `doctrine.repo.preferredremote`. doctrine's own
config namespace — no interop constraint with forgettable's `forget.repo.*` (only
the frame *algorithm* must match byte-for-byte, not config key names).

### F6 — `Anchor` excludes repo identity; the trust pair lives on `Scope` (PHASE-03)

The validated `Anchor` is `git::Frame`'s persisted subset **minus** `repo`
(`RepoIdentity`) plus `verified_sha` + `normalizer`. The repo identity splits:
`repo_id` → the long-standing `scope.repo` string; `repo_id_kind`/`confidence` →
new `Scope` fields (design §5.3). So `Anchor` carries only `kind/commit/tree/
ref_name/checkout_state_id/base_commit/verified_sha/normalizer` — no repo. Don't
re-add repo to Anchor in PHASE-04 render: build `[git]` from the Frame's frame
fields and `[scope]` from `Frame.repo` + scope flags.

Enum string forms are pinned on `git::{AnchorKind,RepoIdKind,Confidence}` as
`parse`/`as_str` (snake_case: `commit`/`checkout_state`/`none`,
`explicit`/`remote`/`local_root`, `high`/`medium`/`low`). PHASE-04 template/render
and PHASE-06 render **must** emit via `as_str` (single source of spelling).

Empty→default is **explicit in `memory.rs` `TryFrom`**, not in `parse` and not
serde: empty/absent `anchor_kind`→`AnchorKind::None`; empty `repo_id_kind`→
`LocalRoot`; empty `repo_id_confidence`→`Low` (lowest-trust, notes F2). `parse`
errors on an unknown non-empty token (covered by tests). `RawReview.review_by` is
carried through to `Memory.review_by` (read by no verb yet; surfaced so the parsed
field is not a dead, never-read struct member — PHASE-05 verify edits it via
`toml_edit`, PHASE-06 may render it).

### F7 — record write path: normalizer + constraint-4 calls (PHASE-04) — AUDIT

Two judgement calls baked into committed output — **confirm at close-out audit**:

1. **`[git].normalizer` = `CHECKOUT_NORMALIZER` iff `anchor_kind=checkout_state`,
   else `""`.** doctrine's flat `[git]` has a single `normalizer` field where
   forgettable puts it on `RepoIdentity` (=`forget.remote.v1`). Chosen to tag the
   only per-record content hash whose algorithm must travel with the anchor (the
   `checkout_state_id`); a clean commit / none anchor leaves it empty (no content
   hash to version). repo-identity algorithm stays implicit in `repo_id_kind` + the
   golden vector. *Risk:* if the reader (SL-008 staleness) or forgettable expects a
   always-present remote tag here, revisit. PHASE-06 `show` only reads presence.

2. **Constraint-4 predicate fires on ANY non-empty `repo_id`, including the
   auto-derived `repo:git-root:unborn` an unborn repo yields.** So a *bare* record
   in an **unborn** git repo (no commits, no remote, no `--repo`) ERRORS
   ("no git anchor"), while a bare record in a **non-git** dir SUCCEEDS unscoped
   (capture → empty `repo_id`). This is the literal design (§5.4 m1: "derived or
   --repo" counts; §5.5: "unborn … a repo-scoped record here errors"). Asymmetry is
   intentional per spec but surprising — flagged. The clean repo-scoped-in-non-git
   error path is exercised via `--repo` in a non-git dir (VT-2a).

**Render seams (ride in PHASE-06):** `render_memory_toml` emits every git/scope
token via `git::*::as_str` (single spelling source). User-influenceable values
(`title`/`summary`/`tags`/scope arrays/`repo`) route through `toml_string` /
`toml_array_inner` (A-1 escaping — proven by `repo_override_with_a_hostile_value`).
`--repo` overrides `frame.repo` via `git::explicit_identity` (kind=explicit/high,
userinfo-stripped). `RecordArgs<'a>` is the shell-input bundle (parallels `Draft`).
Template is **rust-embed**'d from `install/` → a rebuild (not `doctrine install`)
picks up template edits.
