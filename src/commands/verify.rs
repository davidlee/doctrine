// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine verify <check>` — Doctrine's own runbook checks (SL-233 PHASE-16,
//! sketch §4.2).
//!
//! **Not [`crate::verify`].** That module is engine tier and holds
//! coverage-verification config (`.doctrine/adr/001/layering.toml`); it has
//! nothing to do with this family. The two names collide only in the reader's
//! eye, and this comment is the whole reason the collision is survivable.
//!
//! **Why a family at a durable address.** A runbook step's `verify` argv lands
//! in committed TOML, so the address *is* API: moving a shipped check later
//! breaks every downstream runbook that named it. A family gives those checks
//! one place to live and one contract to keep.
//!
//! **The contract is exit code plus explanatory output, and nothing else.** No
//! runbook may key on a verb that exists for another reason — `doctrine slice
//! research` is advisory by construction and returns `Ok(())` on every outcome
//! (ADR-003), so a runbook borrowing it would be reading an exit code that
//! means nothing. That verb also *mints* an absent baseline, which would make
//! the check satisfy itself the first time it ran. A check reads; it never
//! repairs what it is checking.
//!
//! **Closed by intent.** The open-ended half of the verifier contract is a
//! project's own argv (§4); what is closed here is the set of checks Doctrine
//! ships and stands behind. `research-current` is the one PHASE-16 needs; the
//! rest follow the runbooks that want them rather than being speculated into
//! existence.

use std::io::{self, Write};
use std::path::PathBuf;

/// The shipped checks. One variant per durable address.
#[derive(clap::Subcommand, Debug)]
pub(crate) enum VerifyCommand {
    /// The slice's pre-design research round has run and is current.
    ///
    /// Non-zero when no baseline exists (the round has not run) or when the
    /// slice's intent docs have drifted past it (the artefact may be stale).
    ResearchCurrent {
        /// The slice, e.g. `SL-233`.
        #[arg(long)]
        slice: String,

        /// Explicit project root (default: auto-detect from CWD).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

/// Dispatch a check. Every arm reads; none writes.
pub(crate) fn verify(command: VerifyCommand) -> anyhow::Result<()> {
    match command {
        VerifyCommand::ResearchCurrent { slice, path } => research_current(&slice, path),
    }
}

/// `doctrine verify research-current --slice <ref>`.
///
/// Absence and drift are distinct failures and say so: the first means the
/// round was never run, the second that it was run against something the slice
/// has since moved past. Both are non-zero — a step whose verifier is this
/// check is not discharged by either.
/// It deliberately does **not** confirm the parent slice exists first, the way
/// `slice research` does: that verb mints under the slice and so owes the
/// precondition, where this one only reads. A slice that does not exist has not
/// run its research round either, and the message says exactly that.
fn research_current(slice: &str, path: Option<PathBuf>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let id = crate::slice::parse_ref(slice)?;

    let baseline = crate::research::research_dir(&root, id).join(crate::research::BASELINE_FILE);
    if !baseline.exists() {
        anyhow::bail!(
            "SL-{id:03} has no research baseline — the pre-design research round has not run. \
             Run `/research`, then `doctrine slice research {id}` to stamp it."
        );
    }

    let drift = crate::research::check(&root, id)?;
    if !drift.is_empty() {
        let mut lines = vec![format!(
            "SL-{id:03}'s research baseline has drifted — the artefact may be stale:"
        )];
        lines.extend(drift.changed.iter().map(|p| format!("  changed  {p}")));
        lines.extend(drift.added.iter().map(|p| format!("  added    {p}")));
        lines.extend(drift.removed.iter().map(|p| format!("  removed  {p}")));
        lines.push(format!(
            "Refresh the affected research.md sections, then: doctrine slice research {id} --restamp"
        ));
        anyhow::bail!(lines.join("\n"));
    }

    let out = io::stdout();
    writeln!(&out, "Research baseline for SL-{id:03} is current.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: u32 = 233;

    /// A tree holding the slice's scope doc — the one baseline path a
    /// pre-design mint records.
    fn tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let slice = dir
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{ID:03}"));
        std::fs::create_dir_all(&slice).unwrap();
        std::fs::write(slice.join(format!("slice-{ID:03}.md")), "scope").unwrap();
        dir
    }

    fn run(dir: &tempfile::TempDir) -> anyhow::Result<()> {
        research_current("SL-233", Some(dir.path().to_path_buf()))
    }

    /// The round has not run. This is the state the shipped `explore.research`
    /// step exists to catch, so it must be a refusal and not a shrug.
    #[test]
    fn an_unstamped_slice_fails_the_check() {
        let dir = tree();
        let err = run(&dir).expect_err("no baseline is a failure");
        assert!(
            err.to_string().contains("no research baseline"),
            "the failure must name what is missing, said: {err}"
        );
    }

    /// Stamped and unmoved: the only passing state.
    #[test]
    fn a_current_baseline_passes() {
        let dir = tree();
        crate::research::mint(dir.path(), ID, "2026-07-31").unwrap();
        run(&dir).expect("a freshly stamped baseline is current");
    }

    /// Stamped, then the intent moved past it. Distinct from absence, and the
    /// per-path drift is the explanatory output the contract promises.
    #[test]
    fn a_drifted_baseline_fails_and_names_the_path() {
        let dir = tree();
        crate::research::mint(dir.path(), ID, "2026-07-31").unwrap();
        let scope = dir
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{ID:03}"))
            .join(format!("slice-{ID:03}.md"));
        std::fs::write(&scope, "scope, rewritten").unwrap();

        let err = run(&dir).expect_err("drift is a failure");
        let text = err.to_string();
        assert!(
            text.contains("drifted") && text.contains(&format!("slice-{ID:03}.md")),
            "drift must be distinguishable from absence AND name the path: {text}"
        );
    }
}
