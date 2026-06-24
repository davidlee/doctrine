I now have all the evidence. Here is the comprehensive analysis.

---

## ISS-038 Research Verdict

**Status: Partially mitigated — not closed.**

The M4 pre-gate added by SL-121 PHASE-02 (commit `426f024e`, `src/dispatch.rs:1599-1611`) catches the exact ISS-038 scenario for **tracked** dirt — integrate now bails with `integrate-dirty-worktree` before any ref moves. But the ISS-030 phantom has a narrow residual path (None-leg `RacedDesync`), and the `/close` skill still has no recovery procedure.

---

### Q1 — Reproduce the ISS-038 chain against current code

**Evidence.** The original SL-122 incident (`commit 40cc6b8f`, 2026-06-20 16:48) rode a stale binary. The `.doctrine` commit `40cc6b8f` captured the phantom: `git diff 45d0d354..40cc6b8f -- src/rfc.rs` shows 509 lines deleted — the entire SL-122 `src/rfc.rs` was silently reverted by an unrelated `.doctrine` commit that rode the stale index.

**What ran.** The binary at the time of SL-122 close did **not** include SL-121 PHASE-02 (committed at 11:30 same day — the shared `CARGO_TARGET_DIR` likely served a stale binary). The old integrate code (`58f9d806~1:src/dispatch.rs`) used plain `git::replay_ref` — pure CAS `update-ref` with zero worktree awareness, zero pre-gate. That is the code that created the phantom.

**Current-code trace.** With the **current** binary (post SL-121), when `--integrate --trunk refs/heads/main` runs:

| Precondition | Leg executed | Phantom? |
|---|---|---|
| `main` checked out, **tracked-dirty** | Pre-gate **bails** (`dispatch.rs:1607`): `bail!("integrate-dirty-worktree (refs/heads/main)")` | **No** — zero refs moved |
| `main` checked out, **clean** (or untracked-dirty only) | `advance_checked_out` → `ff_advance_in_worktree` → `merge --ff-only` syncs ref+index+tree atomically | **No** — test at `git.rs:3803-3827` proves `git status --porcelain` empty after |
| `main` **not checked out** anywhere | `advance_pure_ref` → `update_ref_cas` → `Disposition::AdvancedPureRef` | **No** — nothing to desync |
| `main` not checked out at probe, **appears checked-out+dirty** in post-CAS re-probe | `advance_pure_ref` → CAS succeeds → re-probe finds dirty checkout → `Disposition::RacedDesync` (`dispatch.rs:1700`) | **YES** — ref advanced, worktree stale, warning emitted |

**Verdict on SL-122 chain reachability today:** The **exact** ISS-038 chain (tracked-dirty checkout → integrate advances ref → phantom) is **not reachable** with the current binary — the M4 pre-gate bails first. But the **phantom outcome** (ref advanced, worktree stale) is still reachable through the None-leg `RacedDesync` path (row 4 above), with **extremely low** likelihood (requires a checkout to materialize between the CAS and post-CAS re-probe, AND be dirty).

**Confidence: HIGH** — verified against git history (`40cc6b8f` phantom commit, `426f024e` pre-gate addition) and current code logic.

---

### Q2 — M4 pre-gate coverage gap

**(a) Untracked concurrent dirt — CONFIRMED GAP.**

`tree_clean` (`git.rs:1176-1179`):
```rust
pub(crate) fn tree_clean(root: &Path) -> Result<bool, CaptureError> {
    let status = git_text(root, &["status", "--porcelain", "--untracked-files=no"])?;
    Ok(status.is_empty())
}
```

The `--untracked-files=no` flag is explicit. The test at `git.rs:3784-3797` confirms: writing `untracked.txt` → `tree_clean` still returns `true`. The comment at `git.rs:1181-1183` justifies this as deliberate: *"Untracked scratch is deliberately excluded — ephemeral files in a close session must not block a clean advance."*

The pre-gate (`dispatch.rs:1599-1611`) calls `tree_clean` → misses untracked-only dirt. This is **by design**, not a bug. For the checked-out leg, untracked collisions are caught by `merge --ff-only`'s own abort (which `ff_advance_in_worktree` converts to `Raced` → safe refusal). For the None leg, untracked dirt is harmless (pure CAS doesn't touch worktrees).

**Gap confirmed, but not a phantom vector for the checked-out leg.**

**(b) Raced-failure-after-advance window (§7) — CONFIRMED GAP, leg-dependent.**

The comment at `dispatch.rs:1603-1604` explicitly acknowledges:
```rust
// Pre-existing dirt only; concurrent dirt is a raced-failure-after-advance (§7).
```

- **Checked-out leg**: `ff_advance_in_worktree` (`git.rs:1209-1271`) has its own `tree_clean` M5 guard at `git.rs:1239` in the probe→merge window. Dirt appearing after the pre-gate but before `merge --ff-only` → guarded → `Raced` → ref NOT advanced. **Window closed.**

- **None leg**: No mid-window guard. `update_ref_cas` (`git.rs:893-908`) runs unconditionally after the pre-gate. The post-CAS re-probe at `dispatch.rs:1693-1701` catches dirt but the ref **already advanced**. `RacedDesync` is a warning, not a refusal. **Window open.**

**Confidence: HIGH** — directly confirmed by code comments and logic.

---

### Q3 — None-leg vs checked-out-leg: which leg did SL-122 hit, and does the checked-out leg's FF fully sync?

**SL-122 hit the OLD code path** (pre-SL-121 `replay_ref`, pure CAS). No leg classification existed. The old code (`58f9d806~1:src/dispatch.rs`) called `git::replay_ref` which does `update-ref` and nothing else — it never synced any checkout.

**Current routing for same conditions.** If `main` is checked out in the shared worktree: `worktree_for_ref` returns `Some(wt)` → `advance_checked_out` → `ff_advance_in_worktree`. The checked-out leg's FF-merge **fully syncs** ref+index+tree (`git.rs:1209-1271` and test at `git.rs:3803-3827`):
```rust
// git.rs:3823-3827 — the test assertion
assert!(
    repo.git(&["status", "--porcelain"]).is_empty(),
    "index + worktree at planned — no phantom reverse-diff",
);
```

**Only the None-leg post-CAS resync has the residual.** When `worktree_for_ref` returns `None` at first probe but `Some(dirty)` at re-probe → `Disposition::RacedDesync` (`dispatch.rs:1700`) → ref advanced, worktree stale → phantom.

**Confidence: HIGH** — direct code + test evidence.

---

### Q4 — IMP-122 hardening status

**Both hardenings are ABSENT from current code.**

**F-1 (re-resolve target_ref before reset --hard):** In `advance_pure_ref` (`dispatch.rs:1674-1708`), after `update_ref_cas` succeeds, the code probes `worktree_for_ref` and `tree_clean`, then calls `resync_worktree_hard(&wt, planned)` without re-verifying that `target_ref` still resolves to `planned`. A concurrent writer could advance `target_ref` past `planned` in the CAS→resync window — `reset --hard planned` would silently clobber that advance.

**F-2 (untracked-collision guard before reset --hard):** `resync_worktree_hard` (`git.rs:1274-1277`) runs `git reset --hard <oid>` with no pre-check for untracked collisions. Unlike `merge --ff-only` (which aborts on collisions), `reset --hard` silently overwrites.

```rust
// git.rs:1274-1277 — no collision guard
pub(crate) fn resync_worktree_hard(wt: &Path, oid: &str) -> Result<(), CaptureError> {
    git_text(wt, &["reset", "--hard", oid])?;
    Ok(())
}
```

**Relationship to ISS-038.** IMP-122 is **genuinely separate** from ISS-038's phantom. ISS-038 = ref advances without syncing checkout (the phantom itself). IMP-122 = hazards that occur **during** the resync attempt after the phantom has already occurred (the `RacedDesync` path) — clobbering a concurrent advance or silently destroying untracked files during `reset --hard`. Both are correct, narrow, race-gated hardenings the SL-121 audit identified. Neither has been implemented.

**Confidence: HIGH** — code absence confirmed by direct reading.

---

### Q5 — /close recovery gap

**The close skill has a detector but NO recovery procedure.**

At `.agents/skills/close/SKILL.md:88-103`:
```bash
# (a) No phantom reverse-diff: the tracked working tree matches HEAD. A nonzero
#     exit means integrate advanced the ref but desynced the live checkout — STOP.
git diff --quiet HEAD
```

The skill says **STOP** but gives zero guidance on what to do next. The ISS-038 issue text's suggested recovery — *"do NOT commit anything (even unrelated files) until the checkout is resynced to the advanced ref"* — is not in the skill.

A search of the entire skill corpus (`.agents/skills/**/*.md`) for `resync`, `restore`, `phantom`, or `recover` finds **only** the `git diff --quiet HEAD` detector in `close/SKILL.md` — no recovery procedure exists anywhere.

**ISS-030 (closed)** captured the same gap: *"Close skill verify... gives no signal when invoked while the trunk branch is checked out... the skill never tells you to sync the worktree."* ISS-030 is marked closed/done, but the recovery gap described in it persists in the current close skill.

**Confidence: HIGH** — confirmed by full-skill-corpus search.

---

### Q6 — Re-baseline ISS-038 issue text

| ISS-038 claim | Status | Evidence |
|---|---|---|
| "Integrate must fail-closed on a dirty trunk checkout" | **Mitigated for tracked dirt** | Pre-gate at `dispatch.rs:1607` bails on tracked dirt; `tree_clean` misses untracked-only |
| "A hard pre-gate... before --integrate would prevent the phantom entirely" | **Implemented (SL-121) but narrow** | The pre-gate exists but only for tracked dirt (`--untracked-files=no`, `git.rs:1177`) |
| "Integrate should operate via a dedicated clean worktree / pure ref CAS" | **Still accurate, unaddressed** | Current code still depends on shared checkout state via `worktree_for_ref`/`tree_clean` |
| "The ISS-030 STOP needs a defined recovery, not just 'STOP'" | **Still accurate, unaddressed** | Close skill has no recovery procedure (Q5 above) |
| "Multi-agent hazard: integrate moves shared main ref while another agent commits" | **Partially accurate** | SL-126's structural close-gate is a belt; mechanism-level hazard persists in None-leg `RacedDesync` |
| "A subsequent .doctrine commit... rode that stale index... silently reverted" | **Still accurate as a class of risk** | The None-leg `RacedDesync` phantom could still be captured by an unwary subsequent commit |

**Confidence: HIGH** — each claim tested against current code.

---

## Final Verdict

**ISS-038: PARTIALLY MITIGATED — not closed.**

The M4 pre-gate (SL-121 PHASE-02, `dispatch.rs:1599-1611`) closes the exact failure chain for **tracked** dirt on the trunk checkout. With current code, the SL-122 phantom cannot recur via the same mechanism.

**Precise residuals (what's still open):**

| # | Residual | Location | Severity |
|---|---|---|---|
| R1 | None-leg `RacedDesync` phantom | `dispatch.rs:1700` → `Disposition::RacedDesync` | **Low likelihood** × **high impact** (would produce the same phantom) |
| R2 | No recovery procedure in `/close` | `.agents/skills/close/SKILL.md:88-103` | **Medium** — the detector fires but the agent is left with no guidance |
| R3 | IMP-122 F-1 missing (re-resolve before `reset --hard`) | `dispatch.rs:1698` → `resync_worktree_hard` | **Low likelihood** (None-leg only) × **medium impact** |
| R4 | IMP-122 F-2 missing (untracked-collision guard) | `git.rs:1276` → `reset --hard` | **Low likelihood** × **low impact** |

**Partition against IMP-122 and RFC-005 OQ-5:**

- **IMP-122** addresses hazards *within* the None-leg resync (after the phantom exists) — separate from ISS-038's phantom-creation mechanism.
- **RFC-005 OQ-5** asks whether integrate should be refactored to be checkout-independent. This is the structural root: if integrate never depends on shared checkout state, the ISS-038 phantom, the None-leg `RacedDesync` residual, and IMP-122's resync hazards all vanish. The current M4 pre-gate is a *guard* on the existing mechanism; OQ-5 asks whether the mechanism itself should change.
