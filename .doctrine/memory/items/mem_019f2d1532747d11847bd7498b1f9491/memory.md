# Recover a dispatch close deadlocked by trunk-side authored divergence via refresh-base, not a trunk reset

**The deadlock.** The sanctioned dispatch close chain is a ladder where every rung
gates the next: `slice status done` refuses without a **journal trunk row** → only
`sync --integrate` writes it → integrate refuses without an admitted `close_target`
→ `admit` refuses without a Doctrine-computed clean `merge_oid` → `candidate create`
runs its own all-or-nothing 3-way merge and records **no** `merge_oid` on any
conflict (see [[mem_019ee4bac0597bf0809caf56b0e59466]]). So a *single* conflicting
path in `review/<N>` vs trunk dead-ends the whole close — hand-resolving the merge
does not help (admit validates the recorded OID, which stays empty).

**The trigger that isn't base drift.** A conflict here need not come from trunk
moving under the bundle. It also fires when trunk carries a **deliberate authored
`.doctrine/**` divergence** the immutable `review/<N>` bundle predates. SL-198: the
owner **truncated** `.doctrine/rfc/011/case-notes.md` (846→33 lines) on trunk on
purpose; the audited bundle still held the 846-line version, so *every* `candidate
create` conflicted on that one file — permanently, since the truncation is intended
to stay.

**The recovery — reconcile the divergence into the coord branch, no trunk reset.**
Do NOT reset trunk backward to a pre-divergence tip (shared-history rewrite; races
other agents). Instead run the ceremony *on the current trunk* so integrate becomes
a near-empty FF whose real job is to write the ledger row:

1. `dispatch refresh-base --slice N` — merges current trunk into `dispatch/<N>` in
   the coord worktree. It **conflicts on the diverged file**; resolve it to trunk's
   version (`git checkout --theirs -- <path>` — trunk is *theirs* when merging trunk
   into coord), `git add`, then complete the merge (`git commit --no-edit`; refuses
   a partial commit mid-merge). Refuses up front over a dirty coord worktree — clear
   it first (`git restore --staged --worktree <ledger paths>` if the journal/
   boundaries show staged-deleted).
2. `dispatch sync --slice N --prepare-review` — re-cut `review/<N>` + `phase/<N>-NN`.
   It **CAS-refuses** to clobber the existing refs ("stale ref(s), not clobbered")
   and has no force flag: `git branch -D review/N phase/N-01 …` first (the commits
   stay reachable via coord history; none are checked out), then re-run.
3. `candidate create --role close_target --base <trunk> --source review/N
   --supersedes <old-conflicted-cand>` — now merges **clean** (if the code already
   landed on trunk, the delta is near-empty).
4. `candidate admit --role close_target --candidate <ref> --review RV-NNN`.
5. `sync --slice N --integrate --trunk <trunk>` — advances trunk (pure-ref if trunk
   isn't checked out; "advanced+pure-ref") and **writes the journal trunk row**.
   Verify (b): `sync --show-journal-trunk-oid --trunk <trunk>` == trunk ref.
6. `slice status N done` → close the originating backlog item.

**Lighter path when the payload already landed — `dispatch sync
--record-integration` (SL-211).** Steps 2–5 re-cut `review/<N>`, rebuild a
candidate, admit, and `--integrate`; but when the reviewed **payload** (the
phase-chain tip / admitted `close_target` — never `review/<N>`, a review surface
SPEC-022 forbids as a trunk payload) is **already an ancestor of trunk** (the
"delta is near-empty" case at step 3 — the code has landed and integrate would
only *record*, not advance), skip that ceremony:

    dispatch sync --slice N --record-integration --trunk <trunk>

resolves the earned payload from the ledger, asserts it already sits on trunk,
and commits the Verified trunk row directly. Reserve the full
candidate→admit→`--integrate` path for when integrate must genuinely **advance**
trunk. **Never hand-write the row** — that was the pre-SL-211 stopgap (IMP-236,
shipped as this verb). See [[mem_019ee4bac0597bf0809caf56b0e59466]].

**Cost consciously paid.** Re-cutting `review/<N>` changes the audited surface's
**ref OID** but not its **code** — the divergent file is instrumentation, not a code
unit, so the audit (RV) stays valid. No trunk reset, no history rewrite.

**Prevention.** Keep append-only instrumentation logs (RFC case-notes) off the
audited code bundle, or truncate them *before* cutting `review/<N>`, not after — a
post-audit trunk-side edit to any authored path the bundle carries re-arms this
deadlock. Related: pre-FF-trunk alternative [[mem_019f06a18bf97b23bf771740e427b639]];
integrate/close landing [[mem_019ec912f7fd746284bfaef00717443e]].
