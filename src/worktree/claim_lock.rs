// SPDX-License-Identifier: GPL-3.0-only
//! The per-name **claim lock** — the OS advisory lock that serialises the fork
//! `claim → bind → act` window against a concurrent same-name spawn AND against
//! gc's branch-residue sweep (SL-228 PHASE-04, design §3 / RV-304 F-2).
//!
//! **Module home (ADR-001).** This is a `worktree` sub-module because the claim
//! protocol is a fork-sequence concern and its three callers are all `worktree`
//! sub-modules: `create`/`fork` take it on the spawn side, `gc` tries it on the
//! sweep side. It imports NOTHING from the crate (std + `rustix` + `anyhow` only),
//! so it adds no edge to the layering graph and sits at the bottom of the
//! `worktree::*` tier by construction.
//!
//! **Why `flock` and not a lease file.** The lock is kernel-mediated and
//! **auto-released on process death**, so a crashed spawn leaves no stale lease to
//! reason about — the residue it leaves behind (a branch, maybe a record, no
//! worktree) sweeps freely on the next gc because its lock is already gone. That
//! property is the whole reason the design picked an OS lock over a lease file:
//! there is no stale-lease problem to solve, by construction. Single-host is
//! already the dispatch topology (ADR-008), which is exactly `flock`'s scope.
//!
//! **The lock file is runtime tier** — `<coord>/.doctrine/state/dispatch/lock/<name>`,
//! a SIBLING of the dispatch record and jail policy: gitignored, disposable, and
//! never provisioned into a worker fork. It is never deleted: the file is a pure
//! lock handle (its CONTENTS mean nothing), and unlinking a locked file races
//! two acquirers onto two different inodes — the classic flock-unlink bug.

use anyhow::Context as _;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

/// Where the per-name claim locks live under the coordination root:
/// `<coord>/.doctrine/state/dispatch/lock/<name>`. A SIBLING subpath to the jail
/// policy and the dispatch record (STD-001 single-source named const).
pub(crate) const LOCK_SUBPATH: &str = ".doctrine/state/dispatch/lock";

/// A HELD claim lock, released exactly when this value drops — including on an unwind,
/// an early `?` return, or process death (kernel-mediated). There is deliberately no
/// `release()` verb: scope IS the lifetime, so no caller can forget to release one.
///
/// `Drop` unlocks EXPLICITLY rather than leaning on the close. The lock lives on the
/// open file description, and `fork` gives a child a descriptor onto the SAME one — so
/// a close-only release stays pending until every duplicate is closed too. That matters
/// here because the lock is deliberately held ACROSS child processes (gc runs a dozen
/// `git` invocations under it): a child forked while the lock is held carries a
/// duplicate until it execs, which would make the release lag by however long that
/// takes. An explicit `LOCK_UN` releases the description itself, so the release is
/// immediate and does not depend on anyone else's timing.
#[derive(Debug)]
pub(crate) struct ClaimLock {
    /// The locked handle. Held open so the lock lives; unlocked explicitly on drop.
    file: File,
}

impl Drop for ClaimLock {
    fn drop(&mut self) {
        // Best-effort: a failed unlock still releases when the description closes, and
        // there is no caller who could act on the error at this point.
        if rustix::fs::flock(&self.file, rustix::fs::FlockOperation::Unlock).is_err() {
            // Nothing to do — the close below is the fallback release.
        }
    }
}

/// The claim-lock file for `name` under `coord` (one owner of the
/// `<LOCK_SUBPATH>/<name>` shape — every acquirer routes through here).
fn lock_path(coord: &Path, name: &str) -> PathBuf {
    coord.join(LOCK_SUBPATH).join(name)
}

/// Open (creating if absent) the claim-lock file for `name`.
fn open_lock_file(coord: &Path, name: &str) -> anyhow::Result<File> {
    let path = lock_path(coord, name);
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("claim lock path {} has no parent", path.display()))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create claim lock dir {}", parent.display()))?;
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("open claim lock {}", path.display()))
}

/// BLOCKING acquire — the SPAWN side (design §3): taken before the branch claim and
/// held across bind and act, so no second spawn for the same name can interleave and
/// no gc sweep can mistake this live claimant's branch for crash residue.
pub(crate) fn acquire(coord: &Path, name: &str) -> anyhow::Result<ClaimLock> {
    let file = open_lock_file(coord, name)?;
    rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive)
        .with_context(|| format!("acquire claim lock for {name}"))?;
    Ok(ClaimLock { file })
}

/// NON-BLOCKING acquire — the gc SWEEP side (design §3): `Ok(None)` means the lock is
/// BUSY, which IS the active-claimant signal (a spawn is mid claim→bind→act), so the
/// sweep skips that name this pass rather than destroying a live claim. A crashed
/// spawn's lock is already kernel-released, so its residue acquires freely here.
pub(crate) fn try_acquire(coord: &Path, name: &str) -> anyhow::Result<Option<ClaimLock>> {
    let file = open_lock_file(coord, name)?;
    match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(Some(ClaimLock { file })),
        // BUSY is not an error: it is the answer (an active claimant holds the name).
        Err(rustix::io::Errno::WOULDBLOCK) => Ok(None),
        Err(e) => Err(anyhow::Error::new(e).context(format!("try claim lock for {name}"))),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;

    // --- VT-1 / VT-4: the lock is REAL — exercised against real flock, never a mock.

    #[test]
    fn a_held_lock_is_busy_to_a_non_blocking_acquirer_and_free_after_drop() {
        // The sweep-side contract in one property: while a claimant holds the name,
        // `try_acquire` answers BUSY (`None`) — not an error, not a steal; once the
        // claimant's lock drops, the same call acquires.
        let tmp = tempfile::tempdir().unwrap();
        let coord = tmp.path();

        let held = acquire(coord, "agent-abc").unwrap();
        assert!(
            try_acquire(coord, "agent-abc").unwrap().is_none(),
            "a held claim is BUSY to the non-blocking sweep"
        );
        // A DIFFERENT name is unaffected — the lock is per-name, not global.
        assert!(
            try_acquire(coord, "agent-other").unwrap().is_some(),
            "the claim lock is per-name: a sibling name is free"
        );

        drop(held);
        assert!(
            try_acquire(coord, "agent-abc").unwrap().is_some(),
            "the name is free once the claimant's lock drops"
        );
    }

    #[test]
    fn the_lock_file_lands_in_the_runtime_tier_beside_its_siblings() {
        let tmp = tempfile::tempdir().unwrap();
        let coord = tmp.path();
        let held = acquire(coord, "agent-abc").unwrap();
        assert!(
            coord.join(LOCK_SUBPATH).join("agent-abc").exists(),
            "the lock file lands under the runtime-tier lock subpath"
        );
        drop(held);
    }

    /// The env var that turns [`lock_holder_child`] from a no-op into a lock holder.
    /// Value: `<coord>\n<name>`.
    const HOLDER_ENV: &str = "DOCTRINE_TEST_CLAIM_LOCK_HOLDER";
    /// What the holder child prints once it genuinely holds the lock.
    const HOLDER_READY: &str = "claim-held";

    /// NOT a test in its own right — the child half of
    /// [`a_crashed_claimants_lock_is_released_by_the_kernel`]. Unset env ⇒ instant no-op,
    /// so a normal `cargo test` run just skips through it.
    ///
    /// It exists because the property under test is *process death*, which a thread
    /// cannot model (threads share this process's fd table) and a lock utility cannot
    /// supply (`flock(1)` is not on PATH here). Re-executing this same binary gives a
    /// REAL separate process taking a REAL kernel lock through the REAL `acquire`.
    #[test]
    fn lock_holder_child() {
        let Ok(spec) = std::env::var(HOLDER_ENV) else {
            return;
        };
        let (coord, name) = spec
            .split_once('\n')
            .expect("holder spec is `<coord>\\n<name>`");
        let held = acquire(Path::new(coord), name).expect("the child takes the claim");
        println!("{HOLDER_READY}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        // Outlive the parent's assertions; the parent kills us mid-sleep.
        std::thread::sleep(std::time::Duration::from_secs(120));
        drop(held);
    }

    /// VT-4: a CRASHED claimant's lock is kernel-released, so the residue it left sweeps
    /// freely — there is no stale lease to age out, which is the whole reason the design
    /// chose an OS lock over a lease file.
    ///
    /// The holder is a re-exec of this binary (see [`lock_holder_child`]): a genuinely
    /// separate process, killed outright with no unwinding and no cooperation. Note it
    /// deliberately does NOT hand the lock over by fd inheritance — clearing `CLOEXEC`
    /// would leak the locked descriptor into every other concurrently-forked child in
    /// this test binary and wedge unrelated tests.
    #[test]
    fn a_crashed_claimants_lock_is_released_by_the_kernel() {
        let tmp = tempfile::tempdir().unwrap();
        let coord = std::fs::canonicalize(tmp.path()).unwrap();

        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "worktree::claim_lock::tests::lock_holder_child",
                "--nocapture",
            ])
            .env(HOLDER_ENV, format!("{}\nagent-crash", coord.display()))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn a real lock-holding child process");

        // Wait until the child reports it genuinely holds the lock.
        let out = child.stdout.take().expect("child stdout");
        let ready = std::io::BufRead::lines(std::io::BufReader::new(out))
            .map_while(Result::ok)
            .any(|line| line.trim() == HOLDER_READY);
        assert!(ready, "the child took the claim");

        assert!(
            try_acquire(&coord, "agent-crash").unwrap().is_none(),
            "the live claimant's lock is BUSY to the sweep"
        );

        // Kill it — the kernel releases the lock with the process.
        child.kill().expect("kill the holder");
        child.wait().expect("reap the holder");
        assert!(
            try_acquire(&coord, "agent-crash").unwrap().is_some(),
            "a crashed claimant's lock is kernel-released — the residue sweeps freely"
        );
    }
}
