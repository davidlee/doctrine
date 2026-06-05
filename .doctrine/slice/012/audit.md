# Audit SL-012 — symlink-tolerance re-sync

Code review of `eac3a4a` (`src/git.rs`, the only code surface). Reviewed against
forgettable DE-010 (`../forgettable/src/git_context.rs`), design.md, and the
conformance contract (DEC-010-06).

**Verdict: acceptable.** Output-conformance holds — golden vector
`conformance_golden_vector` still pins csid `88d9489028e302700c2e6430e6df1d06539dccfd283d2ed99995258482ccf86c`
byte-identical, proving symlink-free csids do not move. 48/48 git:: tests green
(5 new symlink + submodule-reject retained). The sins are fidelity-claim
accuracy and undocumented mirror drift, not bugs.

## Findings

### A-1 🟠 The mirror is half a mirror, and the slice does not say so
`untracked_fingerprint` (`src/git.rs:766`). Forgettable's DE-010 bundled **two**
changes: symlink tolerance **and** IMPR-003 — batching untracked regular-file
hashing into one `git hash-object --stdin-paths` fork (`untracked_hashes`,
`git_context.rs:830`). Doctrine took the symlink half and left
`untracked_fingerprint` forking once per path. Output-equivalent (git blob oid is
fork-count-invariant), so conformance holds — but the commit ("Mirror …
**byte-for-byte**") and design ("mechanical byte-mirror") overstate fidelity at
the source level. The two files are now structurally further apart than before,
and the cut is unrecorded. R2 ("re-sync drift") is the design's own named risk;
this is exactly it, uncaptured. The next re-syncer cannot tell intentional drift
from rot.
- **Disposition:** fix. Soften the claim to "output-conformant (DEC-010-06), not
  source-identical." Record the dropped batching as deliberate deferred drift,
  pointing at forgettable IMPR-003, so a future mirror-er does not "fix" it blind.

### A-2 🟡 Accidental correctness on the newline-in-path edge, unguarded
`untracked_fingerprint` (`src/git.rs:748`). Forgettable's batching forced it to
discover Finding A: a path containing `\n` cannot ride LF-separated
`--stdin-paths`, so it falls back to per-argv. Doctrine is per-argv for **all**
regular files → safe, but by accident, not intent. No test pins the property.
If batching is later ported (per A-1) to "catch up", the newline trap returns
with no red test to stop it. Invisible coupling between "we skipped the
optimization" and "we're safe".
- **Disposition:** optional. A regression test (`untracked file with '\n' in
  name hashes correctly`) would make the safety explicit and survive a future
  batch port.

### A-3 🟡 Coverage gap — tracked-but-dirty symlink
Design §3/§5 explicitly claims a **changed** tracked symlink rides
`worktree_fingerprint` via `diff --binary`. `symlink_repo_captures_clean` only
exercises the clean Commit path. No test repoints a tracked symlink and asserts
CheckoutState/csid. It is git's diff codepath (low risk), but the design's
load-bearing claim is unproven while five tests pile onto the untracked path.
- **Disposition:** fix (cheap). Add a `tracked_symlink_repoint_is_dirty` test —
  commit a symlink, repoint it in the worktree, assert `AnchorKind::CheckoutState`
  and determinism. Closes the design's last unproven claim.

### A-4 🟡 `is_ok_and` swallows lstat failure into the wrong branch (inherited)
`untracked_fingerprint` (`src/git.rs:773`):
`symlink_metadata(&full).is_ok_and(|m| m.file_type().is_symlink())`. If lstat
**errors**, the entry is treated as a regular file and routed to
`git hash-object -- path`, which **follows** the link and hashes target content —
the precise no-follow violation this slice exists to prevent. Byte-identical to
forgettable, so not a regression. But a `CaptureError::Io` variant was added to
dignify a *readlink* failure while an *lstat* failure gets silently demoted to
"not a symlink". Determinism footgun riding under the conformance flag.
- **Disposition:** accept (inherited from upstream; changing it forks the
  mirror). Note for the next forgettable re-sync — fix belongs upstream.

### A-5 🔵 Perf — one fork per untracked file
Flip side of A-1: `git hash-object` is forked once per untracked file vs
forgettable's single batched call. `--exclude-standard` keeps the count low in
the dogfood repo. Minor.
- **Disposition:** accept (tracked via A-1's drift note).

## Good (👍)
- Symlink tests are behavioural, not theatre: content-invariance (pointee placed
  **outside** the repo to avoid self-confounding), repoint-tracking, dangling,
  non-UTF-8 target bytes, double-capture determinism. Mirror forgettable NF-001.
- Golden vector pins the literal csid → DEC-010-06 proven, not asserted.
- Submodule (160000) rejection coverage retained under the rename
  (`submodule_gitlink_entry_is_rejected`).
- Doc comments describe doctrine's actual per-path impl, not forgettable's
  batching prose blindly transcribed.

## Durable harvest
- `mem.pattern.testing.stale-cargo-bin-exe` recorded — stale `CARGO_BIN_EXE`
  build trap (touch `tests/*.rs` to fix e2e spawn-NotFound). Untracked; fold into
  the SL-012 commit.
