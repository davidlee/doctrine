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
