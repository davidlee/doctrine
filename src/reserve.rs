// SPDX-License-Identifier: GPL-3.0-only
//! `reserve` — claim-backend selection for fresh-id allocation (SL-148).
//!
//! [`backend`] is the single seam that resolves which [`Claim`](crate::entity::Claim)
//! backend a Fresh-allocating materialise site uses, and the matching re-fetching
//! scan source ([`ScanSource`]) the claim loop unions into its candidate set. It is
//! the SOLE LocalFs-vs-[`GitRef`] selector: it loads `[reservation]`, performs the
//! reachability fetch, and decides degradation per design D8 (§5.4). Routing the 11
//! Fresh call sites through one helper — rather than a literal `&LocalFs` at each — is
//! what lets the second backend drop in behind a single signature (design §5.2, F-3).
//!
//! Layering (ADR-001): `reserve` is engine. It reaches `entity` (engine, same tier)
//! and the leaf seams `git`/`dtoml` (downward). The interactive D8 y/N prompt is NOT
//! imported (that would be an upward edge to `install` = command); instead the prompt
//! is injected as a `PromptFn` from the command-tier caller (the pure/imperative split
//! — the impurity is passed in), keeping `[reservation]`'s config + `GitRef` inside this
//! one already-classified module so no new `layering.toml` entry is needed (R9).

use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

use crate::entity::{Acquired, Claim, ClaimCtx, LocalFs};
use crate::git;

/// The re-fetching scan source returned alongside the backend — owned so it can
/// outlive [`backend`] and be borrowed `&mut` into `entity::materialise`'s
/// [`crate::entity::ReservedIds`] param across the retry loop. Given the entity
/// tree's local numeric dir ids, returns the FULL candidate set (design EX-4).
pub(crate) type ScanSource = Box<dyn FnMut(&[u32]) -> anyhow::Result<Vec<u32>>>;

/// The injected D8 fallback prompt (design EX-2, Q7). A command-tier caller passes
/// `crate::install::prompt_confirm`; `reserve` only holds the function pointer, so it
/// never imports the command-tier `install` module (no upward layering edge).
pub(crate) type PromptFn = fn(&str) -> anyhow::Result<bool>;

/// The reservation reach: which arbiter linearizes a fresh-id claim (design §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Reach {
    /// Single-tree: the local `mkdir` is the claim (today's behaviour). Pin this
    /// explicitly (`reach = local`) to opt out of cross-clone coordination.
    Local,
    /// Cross-clone: the remote ref CAS is the claim; a fetch failure hard-errors.
    Shared,
    /// Cross-clone when a remote is reachable, else degrade to `Local` with a
    /// one-time stderr signal — but a *configured* remote that fails hard-errors
    /// (D8 fail-closed), the operator opting into local fallback explicitly.
    /// The shipped default (D5): OOTB team coordination, degrading to the
    /// no-remote single-tree path with no stdout change.
    #[default]
    Auto,
}

/// The `[reservation]` table of `doctrine.toml` (design §5.2, EX-2). An absent table
/// is `Default` (reach = auto, no remote): with no remote configured, `auto` degrades
/// to the single-tree `Local` path at resolve time, so a repo with no `[reservation]`
/// and no remote produces byte-identical stdout to before — only a one-time stderr
/// signal differs (POL-002 back-compat, §5.4). Parsed LAZILY here, inside the
/// engine-tier consumer (the estimation lazy-projection precedent) — never eagerly in
/// `dtoml::parse`, which would force a `leaf → engine` import.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct ReservationConfig {
    /// The reach. Ships `auto` (D5, PHASE-05): cross-clone coordination when a remote
    /// is reachable, degrading to single-tree `Local` otherwise. Pin `reach = local`
    /// to opt out of remote coordination entirely.
    pub(crate) reach: Reach,
    /// Optional explicit remote name; else `git::resolve_remote` (preferred →
    /// origin → sole).
    pub(crate) remote: Option<String>,
    /// Operator pre-opt-in to local fallback on an `auto` configured-remote failure
    /// (the non-interactive equivalent of accepting the D8 y/N prompt). Default false.
    pub(crate) allow_local_fallback: bool,
}

/// The outer shape projecting just the `[reservation]` table out of a `doctrine.toml`
/// body — tolerant of every other top-level key (mirrors `dtoml`'s tolerant parse).
#[derive(Debug, Default, Deserialize)]
struct ReservationDoc {
    #[serde(default)]
    reservation: ReservationConfig,
}

/// Project the `[reservation]` config from a `doctrine.toml` body (PURE). The
/// engine-tier consumer's own focused reader — keeps `[reservation]` off
/// [`crate::dtoml::DoctrineToml`] so no `leaf → engine` import is forced (R9).
fn parse_reservation_config(text: &str) -> anyhow::Result<ReservationConfig> {
    let doc: ReservationDoc = toml::from_str(text)?;
    Ok(doc.reservation)
}

/// Load the `[reservation]` config under `root` — reuses the shared `dtoml`
/// file-read seam ([`crate::dtoml::read_doctrine_toml_text`]); an absent file is the
/// default (reach = auto; degrades to `Local` with no remote, §5.4/EX-5).
fn load_reservation_config(root: &Path) -> anyhow::Result<ReservationConfig> {
    match crate::dtoml::read_doctrine_toml_text(root)? {
        Some(text) => parse_reservation_config(&text)
            .with_context(|| "Failed to parse [reservation] in doctrine.toml".to_owned()),
        None => Ok(ReservationConfig::default()),
    }
}

/// Env override for the D8 fallback opt-in: `DOCTRINE_RESERVATION_FALLBACK=1` accepts
/// local fallback non-interactively (design §5.4).
const ENV_FALLBACK: &str = "DOCTRINE_RESERVATION_FALLBACK";

/// Read whether the env opt-in is set (`=1`).
fn env_fallback_optin() -> bool {
    std::env::var_os(ENV_FALLBACK).is_some_and(|v| v == std::ffi::OsStr::new("1"))
}

/// The reservation ref namespace root. `<prefix>` keys the canonical id-space
/// (`SL`/`ASM`/… — F-V7), NOT the shared file-stem.
const RESERVATION_REF_PREFIX: &str = "refs/doctrine/reservation";
/// The glob refspec the scan re-fetches every retry (design §5.3).
const RESERVATION_REFSPEC: &str = "+refs/doctrine/reservation/*:refs/doctrine/reservation/*";

// ---------------------------------------------------------------------------
// GitRef backend
// ---------------------------------------------------------------------------

/// The cross-clone reservation backend: the claim linearizes at the remote via a
/// zero-oid create CAS over `refs/doctrine/reservation/{prefix}/{id:03}` (design
/// §5.2/EX-1). `prefix`/`root`/`remote`/`holder` are captured here at construction
/// (only `id` varies per retry, so only it rides `ClaimCtx`, D1/D9).
struct GitRef {
    root: std::path::PathBuf,
    prefix: String,
    remote: String,
    holder_name: String,
    holder_email: String,
}

impl GitRef {
    /// The reservation ref for candidate `id`: `refs/doctrine/reservation/<prefix>/<NNN>`.
    fn refname(&self, id: u32) -> String {
        format!("{RESERVATION_REF_PREFIX}/{}/{id:03}", self.prefix)
    }
}

impl Claim for GitRef {
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired> {
        let refname = self.refname(ctx.id);
        // Canonical ref as the commit message (e.g. `SL-148`).
        let canonical = format!("{}-{:03}", self.prefix, ctx.id);
        // DANGLING empty-tree commit with the holder identity set explicitly (F-2).
        let new_oid = git::commit_empty_tree_as(
            &self.root,
            &canonical,
            &self.holder_name,
            &self.holder_email,
        )
        .with_context(|| format!("Failed to build reservation commit for {canonical}"))?;
        // Push BY OID under a zero-oid create CAS (I4 — no local ref advanced pre-push).
        match git::push_ref_cas(&self.root, &self.remote, &refname, &new_oid, git::ZERO_OID)
            .with_context(|| format!("Failed to push reservation {refname}"))?
        {
            git::RefCas::Updated => {
                // Same-machine exclusion + keeps the loop's H2 cleanup valid (D1).
                match std::fs::create_dir(ctx.dir) {
                    Ok(()) => Ok(Acquired::Won),
                    // E1 split-state (remote won, local dir already exists / foreign):
                    // hard error with the reseat hint, never orphan silently (R3).
                    Err(_) => Err(anyhow::anyhow!(
                        "reservation {canonical} pushed to the remote but its local dir \
                         {} could not be created (split state). Run `doctrine reseat {canonical}` \
                         and pick another id.",
                        ctx.dir.display()
                    )),
                }
            }
            // A rival created the ref first — lost the race; recompute and retry.
            git::RefCas::Moved { .. } => Ok(Acquired::AlreadyHeld),
        }
    }

    #[cfg(test)]
    fn is_remote(&self) -> bool {
        true
    }
}

/// Build the `GitRef` scan source: re-fetch the reservation namespace and union the
/// remote ids with the passed local dirs each call (design EX-4, F-V6).
fn gitref_scan_source(root: &Path, remote: &str) -> ScanSource {
    let root = root.to_path_buf();
    let remote = remote.to_owned();
    Box::new(move |local: &[u32]| {
        // Re-fetch so a rival's post-`AlreadyHeld` ref widens this iteration's set.
        git::fetch_refspec(&root, &remote, RESERVATION_REFSPEC)
            .with_context(|| format!("Failed to fetch reservations from {remote}"))?;
        let mut ids: Vec<u32> = local.to_vec();
        ids.extend(remote_reservation_ids(&root)?);
        Ok(ids)
    })
}

/// The reserved ids visible in the fetched LOCAL reservation namespace — parse the
/// trailing `<NNN>` of every `refs/doctrine/reservation/<prefix>/<NNN>` (design §5.3).
/// Unparseable ref names under the namespace are ignored, not fatal (E3).
fn remote_reservation_ids(root: &Path) -> anyhow::Result<Vec<u32>> {
    let rows = git::for_each_ref(root, &format!("{RESERVATION_REF_PREFIX}/"))
        .context("Failed to enumerate reservation refs")?;
    Ok(rows
        .iter()
        .filter_map(|r| r.refname.rsplit('/').next())
        .filter_map(|seg| seg.parse::<u32>().ok())
        .collect())
}

/// The identity scan source for the `LocalFs` backend: the candidate set is exactly
/// the local dirs (today's behaviour, EX-5).
fn local_scan_source() -> ScanSource {
    Box::new(|local: &[u32]| Ok(local.to_vec()))
}

// ---------------------------------------------------------------------------
// Backend selection (resolve_backend — the sole selector, design EX-3)
// ---------------------------------------------------------------------------

/// Resolve the claim backend + scan source for a fresh-id allocation under `root`,
/// for the kind whose canonical id-space is `prefix` (`SL`/`ASM`/… — the reservation
/// ref segment, F-V7). Loads `[reservation]`, then delegates to [`resolve_backend`]
/// — the SOLE LocalFs-vs-GitRef selector (design EX-3). `prompt` injects the D8 y/N
/// confirmation (the command-tier caller passes `install::prompt_confirm`).
pub(crate) fn backend(
    root: &Path,
    prefix: &str,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    let cfg = load_reservation_config(root)?;
    resolve_backend(root, prefix, &cfg, prompt)
}

/// The SOLE LocalFs-vs-GitRef selector / reachability probe / degradation decider
/// (design EX-3, D8). The reachability fetch *is* the probe; its ids seed the `GitRef`
/// scan. Degradation:
/// - `local` ⇒ `LocalFs`, the remote is never touched (EX-5).
/// - `shared` ⇒ `GitRef`; a fetch failure hard-errors, no fallback (shared is shared).
/// - `auto` + **no remote configured** ⇒ `LocalFs` + a one-time stderr signal (the
///   genuine single-tree fallback).
/// - `auto` + **configured remote that fails** ⇒ hard error by default; the operator
///   opts into local fallback per allocation via the env opt-in / config
///   `allow_local_fallback` / the interactive y/N `prompt` (TTY) — on accept ⇒
///   `LocalFs` + the one-time signal.
fn resolve_backend(
    root: &Path,
    prefix: &str,
    cfg: &ReservationConfig,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    match cfg.reach {
        Reach::Local => Ok((Box::new(LocalFs), local_scan_source())),
        Reach::Shared => {
            let remote = require_remote(root, cfg, "shared")?;
            // Reachability probe (this fetch is also the GitRef scan's first fetch).
            probe_reachability(root, &remote).with_context(|| {
                format!("reach=shared: reservation remote {remote} unreachable")
            })?;
            Ok(gitref(root, prefix, &remote))
        }
        Reach::Auto => resolve_auto(root, prefix, cfg, prompt),
    }
}

/// The `auto` degradation decision (D8).
fn resolve_auto(
    root: &Path,
    prefix: &str,
    cfg: &ReservationConfig,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    let Some(remote) = configured_remote(root, cfg)? else {
        // Structurally single-tree: the genuine PRD-005 fallback case.
        signal_local_fallback("no remote configured");
        return Ok((Box::new(LocalFs), local_scan_source()));
    };
    match probe_reachability(root, &remote) {
        Ok(()) => Ok(gitref(root, prefix, &remote)),
        Err(e) => {
            // Configured remote that FAILS: fail-closed unless the operator opts in.
            if env_fallback_optin() || cfg.allow_local_fallback || prompt_fallback(&remote, prompt)?
            {
                signal_local_fallback(&format!("remote {remote} unreachable: {e}"));
                Ok((Box::new(LocalFs), local_scan_source()))
            } else {
                Err(e).with_context(|| {
                    format!(
                        "reach=auto: reservation remote {remote} unreachable and local fallback \
                         declined. Set [reservation] allow-local-fallback=true or \
                         {ENV_FALLBACK}=1 to allocate locally."
                    )
                })
            }
        }
    }
}

/// Construct the `GitRef` backend + its re-fetching scan source for `remote`.
fn gitref(root: &Path, prefix: &str, remote: &str) -> (Box<dyn Claim>, ScanSource) {
    let (holder_name, holder_email) = git::resolve_holder(root);
    let backend = GitRef {
        root: root.to_path_buf(),
        prefix: prefix.to_owned(),
        remote: remote.to_owned(),
        holder_name,
        holder_email,
    };
    (Box::new(backend), gitref_scan_source(root, remote))
}

/// Resolve the configured remote (explicit `[reservation] remote` else
/// `git::resolve_remote`), `None` when none is configured.
fn configured_remote(root: &Path, cfg: &ReservationConfig) -> anyhow::Result<Option<String>> {
    if let Some(explicit) = &cfg.remote {
        return Ok(Some(explicit.clone()));
    }
    Ok(git::resolve_remote(root)?)
}

/// As [`configured_remote`] but a missing remote is a hard error (for `shared`).
fn require_remote(root: &Path, cfg: &ReservationConfig, reach: &str) -> anyhow::Result<String> {
    configured_remote(root, cfg)?.with_context(|| {
        format!("reach={reach}: no remote configured for reservation coordination")
    })
}

/// The reachability probe: fetch the reservation namespace once. Success means the
/// remote is reachable AND the local namespace now reflects it.
fn probe_reachability(root: &Path, remote: &str) -> anyhow::Result<()> {
    git::fetch_refspec(root, remote, RESERVATION_REFSPEC).map_err(anyhow::Error::from)
}

/// Prompt the operator for the D8 local-fallback opt-in (stderr-only — never stdout,
/// protecting byte-identical CLI output / the behaviour gate).
fn prompt_fallback(remote: &str, prompt: PromptFn) -> anyhow::Result<bool> {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Ok(false); // non-interactive: only the env / config opt-in applies.
    }
    // Prompt to STDERR (behaviour gate — stdout stays byte-identical).
    drop(write!(
        std::io::stderr(),
        "reservation remote {remote} is unreachable. Allocate this id locally (reduced reach)? [y/N] "
    ));
    prompt("")
}

/// Emit the one-time-per-process stderr signal that reach degraded to local — never
/// stdout (behaviour gate). The "one-time" guard is per process via an atomic flag.
fn signal_local_fallback(reason: &str) {
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    static SIGNALLED: AtomicBool = AtomicBool::new(false);
    if !SIGNALLED.swap(true, Ordering::Relaxed) {
        drop(writeln!(
            std::io::stderr(),
            "doctrine: reservation reach degraded to local ({reason})"
        ));
    }
}

// ---------------------------------------------------------------------------
// Held-claims survey (READ path — doctrine reservation list, PHASE-04, REQ-022)
// ---------------------------------------------------------------------------

/// One held reservation as the survey reports it (design §5.2 — the `{canonical,
/// holder, acquired}` table). A plain row struct: no clap, no stdout, no rendering
/// (engine tier). `acquired` is **best-effort client-declared** metadata (the date the
/// holder set on the reservation commit, F-12), NOT a server-attested clock — the
/// command-tier renderer documents that for the operator (EX-3/VA-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeldClaim {
    /// Canonical id derived from the ref path (`…/<prefix>/<NNN>` → `<PREFIX>-<NNN>`,
    /// e.g. `SL-148`).
    pub(crate) canonical: String,
    /// The holder's declared git identity (the reservation commit's author name).
    pub(crate) holder: String,
    /// The holder's declared acquisition time (commit author date) — best-effort,
    /// client-set (F-12).
    pub(crate) acquired: String,
}

/// Survey the held reservations under `root` against `remote` (design §5.4 — fetch →
/// `for_each_ref` → render): re-fetch `refs/doctrine/reservation/*` so the local
/// namespace reflects the remote, enumerate it, and parse each ref into a [`HeldClaim`]
/// (EX-1). `kind` (a canonical id-space prefix segment, e.g. `SL` — F-V7) narrows the
/// result; `None` lists every kind (EX-2). Ref names under the namespace that do not
/// match `…/<prefix>/<NNN>` are SKIPPED, not fatal (E3). Engine tier — no stdout/clock;
/// the command tier resolves the remote, calls this, and renders.
pub(crate) fn survey(
    root: &Path,
    remote: &str,
    kind: Option<&str>,
) -> anyhow::Result<Vec<HeldClaim>> {
    git::fetch_refspec(root, remote, RESERVATION_REFSPEC)
        .with_context(|| format!("Failed to fetch reservations from {remote}"))?;
    let rows = git::for_each_ref(root, &format!("{RESERVATION_REF_PREFIX}/"))
        .context("Failed to enumerate reservation refs")?;
    Ok(rows
        .iter()
        .filter_map(parse_held_claim)
        .filter(|h| kind.is_none_or(|k| held_prefix(&h.canonical) == k))
        .collect())
}

/// Parse a [`crate::git::RefRow`] under the reservation namespace into a [`HeldClaim`].
/// Returns `None` for any ref whose trailing path is not `<prefix>/<NNN>` — a
/// malformed / out-of-band ref the survey SKIPS (E3). `<prefix>` is upper-cased into
/// the canonical id (`sl/001` → `SL-001`); a non-numeric `<NNN>` segment is rejected.
fn parse_held_claim(row: &crate::git::RefRow) -> Option<HeldClaim> {
    let tail = row
        .refname
        .strip_prefix(RESERVATION_REF_PREFIX)?
        .strip_prefix('/')?;
    let mut segs = tail.rsplit('/');
    let num = segs.next()?;
    let prefix = segs.next()?;
    // The id segment must be numeric; a non-numeric one is out-of-band (E3).
    let id: u32 = num.parse().ok()?;
    // `prefix` must be a single id-space segment, not a deeper sub-path or empty.
    if segs.next().is_some() || prefix.is_empty() {
        return None;
    }
    Some(HeldClaim {
        canonical: format!("{}-{id:03}", prefix.to_ascii_uppercase()),
        holder: row.author.clone(),
        acquired: row.date.clone(),
    })
}

/// The id-space prefix of a canonical id (`SL-148` → `SL`) — the `--kind` match key.
fn held_prefix(canonical: &str) -> &str {
    canonical.split('-').next().unwrap_or(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The reservation refspec / namespace pins (the wiring the GitRef e2e relies on).
    #[test]
    fn reservation_namespace_constants() {
        assert_eq!(RESERVATION_REF_PREFIX, "refs/doctrine/reservation");
        assert_eq!(
            RESERVATION_REFSPEC,
            "+refs/doctrine/reservation/*:refs/doctrine/reservation/*"
        );
    }

    // --- ReservationConfig parse (EX-2/EX-5) -------------------------------

    #[test]
    fn absent_table_defaults_to_auto_no_remote() {
        // PHASE-05/EX-1: the shipped default is `auto` (D5). With no remote it
        // degrades to the single-tree `Local` path at resolve time, so EX-5
        // back-compat (byte-identical stdout) holds via §5.4 degradation, not via
        // the parsed reach value.
        let cfg = ReservationConfig::default();
        assert_eq!(cfg.reach, Reach::Auto);
        assert_eq!(cfg.remote, None);
        assert!(!cfg.allow_local_fallback);
        // A body with no [reservation] table → default (tolerant of other keys).
        assert_eq!(
            parse_reservation_config("[dispatch]\ndeliver-to = \"x\"\n").unwrap(),
            ReservationConfig::default()
        );
    }

    #[test]
    fn explicit_reach_local_still_pins_single_tree() {
        // EX-1: an explicit `reach = local` pins single-tree, overriding the new
        // `auto` default — the opt-out remains available.
        let cfg = parse_reservation_config("[reservation]\nreach = \"local\"\n")
            .expect("parse explicit local");
        assert_eq!(cfg.reach, Reach::Local);
    }

    #[test]
    fn reservation_table_parses_tolerantly() {
        let cfg = parse_reservation_config(
            "[dispatch]\ndeliver-to = \"x\"\n\
             [reservation]\nreach = \"auto\"\nremote = \"fork\"\nallow-local-fallback = true\n",
        )
        .expect("parse reservation");
        assert_eq!(cfg.reach, Reach::Auto);
        assert_eq!(cfg.remote.as_deref(), Some("fork"));
        assert!(cfg.allow_local_fallback);
    }

    #[test]
    fn reach_tokens_round_trip() {
        for (tok, reach) in [
            ("local", Reach::Local),
            ("shared", Reach::Shared),
            ("auto", Reach::Auto),
        ] {
            let cfg = parse_reservation_config(&format!("[reservation]\nreach = \"{tok}\"\n"))
                .expect("parse reach");
            assert_eq!(cfg.reach, reach);
        }
    }

    #[test]
    fn unknown_reach_is_an_error() {
        let err = parse_reservation_config("[reservation]\nreach = \"global\"\n").unwrap_err();
        assert!(
            err.to_string().contains("reach"),
            "error names the key: {err}"
        );
    }

    // --- env opt-in -------------------------------------------------------

    #[test]
    fn env_fallback_constant_is_stable() {
        // `set_var` is banned crate-wide; the env branch is proven e2e via the
        // integration tests that drive `backend` with the var set in the child.
        assert_eq!(ENV_FALLBACK, "DOCTRINE_RESERVATION_FALLBACK");
    }

    // --- local backend identity scan (EX-5) -------------------------------

    #[test]
    fn local_scan_source_is_identity() {
        let mut scan = local_scan_source();
        assert_eq!(scan(&[1, 2, 5]).unwrap(), vec![1, 2, 5]);
        assert_eq!(scan(&[]).unwrap(), Vec::<u32>::new());
    }

    // -----------------------------------------------------------------------
    // GitRef e2e against a local bare-remote substrate (jail-safe, NO network).
    // A `git init --bare` remote + working clones referenced by EXPLICIT path, so
    // `.git/config` is never mutated (design D4, R5).
    // -----------------------------------------------------------------------

    use std::path::PathBuf;
    use std::process::Command;

    /// A never-y prompt: declines local fallback (the default D8 posture).
    fn decline(_p: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn git(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00 +0000")
            .output()
            .expect("spawn git")
    }

    fn git_ok(dir: &Path, args: &[&str]) {
        let out = git(dir, args);
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// A bare remote plus N working clones, all under temp dirs, referenced by path.
    struct Substrate {
        _remote: tempfile::TempDir,
        remote_path: PathBuf,
        _clones: Vec<tempfile::TempDir>,
        clone_paths: Vec<PathBuf>,
    }

    impl Substrate {
        fn new(clones: usize) -> Self {
            let remote = tempfile::tempdir().expect("remote dir");
            let remote_path = remote.path().to_path_buf();
            assert!(
                Command::new("git")
                    .args(["init", "--bare", "-b", "main"])
                    .arg(&remote_path)
                    .output()
                    .expect("init bare")
                    .status
                    .success()
            );
            let mut _clones = Vec::new();
            let mut clone_paths = Vec::new();
            for i in 0..clones {
                let c = tempfile::tempdir().expect("clone dir");
                let p = c.path().to_path_buf();
                git_ok(&p, &["init", "-b", "main"]);
                git_ok(&p, &["config", "user.name", &format!("Agent {i}")]);
                git_ok(
                    &p,
                    &["config", "user.email", &format!("agent{i}@doctrine.test")],
                );
                std::fs::write(p.join("seed.txt"), "seed").unwrap();
                git_ok(&p, &["add", "seed.txt"]);
                git_ok(&p, &["commit", "-m", "seed"]);
                _clones.push(c);
                clone_paths.push(p);
            }
            Self {
                _remote: remote,
                remote_path,
                _clones,
                clone_paths,
            }
        }

        fn remote(&self) -> &str {
            self.remote_path.to_str().unwrap()
        }

        fn clone(&self, i: usize) -> &Path {
            &self.clone_paths[i]
        }

        /// Write a `doctrine.toml` with a `[reservation]` table into clone `i`.
        fn write_config(&self, i: usize, body: &str) {
            std::fs::write(self.clone(i).join("doctrine.toml"), body).unwrap();
        }
    }

    /// VT-1: collision-freedom under contention. Two clones compute the SAME
    /// candidate id; exactly one create-push lands; the loser re-fetches, recomputes,
    /// and lands the next id — no duplicate holder (I1, REQ-020/021).
    #[test]
    fn vt1_two_clones_racing_the_same_id_do_not_collide() {
        let env = Substrate::new(2);

        // Clone 0 reserves id 1 (the first candidate over an empty namespace).
        let (b0, _s0) = gitref(env.clone(0), "TK", env.remote());
        let dir0 = env.clone(0).join("tree/001");
        std::fs::create_dir_all(env.clone(0).join("tree")).unwrap();
        let won0 = b0.claim(&ClaimCtx { dir: &dir0, id: 1 }).unwrap();
        assert!(matches!(won0, Acquired::Won), "first clone wins id 1");

        // Clone 1 computes the same candidate (1) — its create-push must be rejected
        // (the ref already exists on the remote): a lost race, not a duplicate.
        let (b1, mut s1) = gitref(env.clone(1), "TK", env.remote());
        std::fs::create_dir_all(env.clone(1).join("tree")).unwrap();
        let dir1a = env.clone(1).join("tree/001");
        let lost = b1.claim(&ClaimCtx { dir: &dir1a, id: 1 }).unwrap();
        assert!(
            matches!(lost, Acquired::AlreadyHeld),
            "second clone loses id 1"
        );

        // The loser re-fetches (the scan source) and recomputes: now id 1 is held
        // remotely, so the next candidate is 2.
        let union = s1(&[]).unwrap();
        let next = crate::entity::next_id(&union, &[]);
        assert_eq!(next, 2, "recompute lands the NEXT free id");
        let dir1b = env.clone(1).join("tree/002");
        let won1 = b1.claim(&ClaimCtx { dir: &dir1b, id: 2 }).unwrap();
        assert!(matches!(won1, Acquired::Won), "second clone lands id 2");

        // Exactly one ref per id on the remote — no duplicate holder.
        let rows = git::for_each_ref(&env.remote_path, "refs/doctrine/reservation/TK/")
            .expect("for_each_ref");
        let mut ids: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.refname.rsplit('/').next())
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, vec!["001", "002"], "one ref each for ids 1 and 2");
    }

    /// VT-4 (e2e): the reservation commit's tree is the empty tree (no blobs); the
    /// entity record carries no coordination bytes (REQ-024, I2). The empty-tree
    /// content-freedom is asserted at the git layer; here we confirm the GitRef claim
    /// path produces it.
    #[test]
    fn vt4_gitref_claim_is_content_free() {
        let env = Substrate::new(1);
        let (b, _s) = gitref(env.clone(0), "SL", env.remote());
        std::fs::create_dir_all(env.clone(0).join("tree")).unwrap();
        let dir = env.clone(0).join("tree/148");
        assert!(matches!(
            b.claim(&ClaimCtx { dir: &dir, id: 148 }).unwrap(),
            Acquired::Won
        ));
        let rows = git::for_each_ref(&env.remote_path, "refs/doctrine/reservation/SL/148")
            .expect("for_each_ref");
        assert_eq!(rows.len(), 1);
        // The ref's commit tree is the empty tree on the remote.
        let tree = git::git_text(
            &env.remote_path,
            &["rev-parse", &format!("{}^{{tree}}", rows[0].oid)],
        )
        .expect("rev-parse tree");
        assert_eq!(tree, git::EMPTY_TREE_OID);
    }

    /// VT-2: reach selection. `local` never touches the remote; `shared` uses it and
    /// hard-fails when the remote is absent; `auto` uses it when reachable.
    #[test]
    fn vt2_reach_selection() {
        let env = Substrate::new(1);
        let root = env.clone(0);

        // local: never touches the remote — a bogus remote is irrelevant.
        env.write_config(
            0,
            "[reservation]\nreach = \"local\"\nremote = \"/no/such/remote\"\n",
        );
        let (b, _s) = backend(root, "TK", decline).expect("local backend");
        assert!(
            !b.is_remote(),
            "local backend must be LocalFs (no remote contact)"
        );

        // shared with an unreachable remote: hard error (no fallback).
        env.write_config(
            0,
            "[reservation]\nreach = \"shared\"\nremote = \"/no/such/remote\"\n",
        );
        assert!(
            backend(root, "TK", decline).is_err(),
            "shared + absent remote hard-errors"
        );

        // shared with a reachable remote: GitRef.
        env.write_config(
            0,
            &format!(
                "[reservation]\nreach = \"shared\"\nremote = \"{}\"\n",
                env.remote()
            ),
        );
        let (b, _s) = backend(root, "TK", decline).expect("shared backend");
        assert!(b.is_remote(), "shared + reachable remote selects GitRef");

        // auto with a reachable remote: GitRef.
        env.write_config(
            0,
            &format!(
                "[reservation]\nreach = \"auto\"\nremote = \"{}\"\n",
                env.remote()
            ),
        );
        let (b, _s) = backend(root, "TK", decline).expect("auto backend");
        assert!(b.is_remote(), "auto + reachable remote selects GitRef");
    }

    /// VT-3 / EX-3: `auto` + **no remote configured** degrades to LocalFs (the genuine
    /// single-tree fallback); `auto` + a **configured remote that fails** hard-errors by
    /// default (D8 fail-closed) and accepts local fallback only on explicit opt-in.
    #[test]
    fn vt3_auto_degradation_is_fail_closed_with_explicit_optin() {
        let env = Substrate::new(1);
        let root = env.clone(0);

        // auto + no remote configured (and none in .git/config) ⇒ LocalFs.
        env.write_config(0, "[reservation]\nreach = \"auto\"\n");
        let (b, _s) = backend(root, "TK", decline).expect("auto no-remote backend");
        assert!(!b.is_remote(), "auto + no remote ⇒ LocalFs");

        // auto + a configured remote that FAILS, prompt declines ⇒ hard error.
        env.write_config(
            0,
            "[reservation]\nreach = \"auto\"\nremote = \"/no/such/remote\"\n",
        );
        assert!(
            backend(root, "TK", decline).is_err(),
            "auto + failing configured remote hard-errors when fallback declined"
        );

        // Same, but config opt-in (allow-local-fallback) ⇒ LocalFs (never silent).
        env.write_config(
            0,
            "[reservation]\nreach = \"auto\"\nremote = \"/no/such/remote\"\nallow-local-fallback = true\n",
        );
        let (b, _s) = backend(root, "TK", decline).expect("opt-in fallback backend");
        assert!(!b.is_remote(), "explicit opt-in ⇒ LocalFs fallback");
    }

    /// PHASE-05 R4 / EX-2: the shipped default (`auto`, no `[reservation]`) in a bare
    /// NON-git directory is structurally single-tree — it degrades to `LocalFs`, never
    /// hard-errors on the absent git repo. This is the exact regression the default-flip
    /// exposed: every entity-creation unit test runs in a bare `TempDir`, so the auto
    /// degradation (§5.4 "no remote configured ⇒ LocalFs") must tolerate a non-repo root
    /// and keep stdout byte-identical (the remote enumeration is short-circuited, not run).
    #[test]
    fn vt2_default_auto_in_a_non_git_dir_degrades_to_localfs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let (b, _s) = backend(tmp.path(), "TK", decline).expect("auto non-git ⇒ LocalFs");
        assert!(
            !b.is_remote(),
            "default auto in a non-git dir must degrade to LocalFs, not error"
        );
    }

    /// VT-5: E1 split-state (remote-won / local-mkdir-failed) hard-errors with the
    /// `doctrine reseat <canonical>` remediation (D6/R3) — no silent orphan.
    #[test]
    fn vt5_split_state_hard_errors_with_reseat_hint() {
        let env = Substrate::new(1);
        let (b, _s) = gitref(env.clone(0), "SL", env.remote());
        // Pre-create the local dir as a FILE so create_dir fails after the push wins.
        std::fs::create_dir_all(env.clone(0).join("tree")).unwrap();
        let dir = env.clone(0).join("tree/009");
        std::fs::write(&dir, "squat").unwrap(); // a file squats the dir path
        let err = b.claim(&ClaimCtx { dir: &dir, id: 9 }).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("doctrine reseat SL-009"),
            "reseat hint present: {msg}"
        );
        // The remote ref still landed (a harmless permanent gap, not rolled back, R3).
        let rows = git::for_each_ref(&env.remote_path, "refs/doctrine/reservation/SL/009")
            .expect("for_each_ref");
        assert_eq!(
            rows.len(),
            1,
            "remote ref is NOT rolled back (harmless gap)"
        );
    }

    /// VT-6 (back-compat seam): a `local` backend never contacts a remote and its
    /// scan is the identity — the materialise loop behaves bit-for-bit as today.
    #[test]
    fn vt6_local_backend_is_back_compatible() {
        let env = Substrate::new(1);
        env.write_config(0, ""); // no [reservation] table at all
        let (b, mut s) = backend(env.clone(0), "TK", decline).expect("default backend");
        assert!(!b.is_remote(), "no [reservation] ⇒ LocalFs (EX-5)");
        // The scan source is the identity (no remote union).
        assert_eq!(s(&[3, 7]).unwrap(), vec![3, 7]);
    }

    // -----------------------------------------------------------------------
    // PHASE-04: held-claims survey (doctrine reservation list, REQ-022).
    // -----------------------------------------------------------------------

    /// Reserve `id` under `prefix` from clone `c` so the survey has something to read.
    fn hold(env: &Substrate, c: usize, prefix: &str, id: u32) {
        let (b, _s) = gitref(env.clone(c), prefix, env.remote());
        let tree = env.clone(c).join("tree");
        std::fs::create_dir_all(&tree).unwrap();
        let dir = tree.join(format!("{id:03}"));
        assert!(
            matches!(b.claim(&ClaimCtx { dir: &dir, id }).unwrap(), Acquired::Won),
            "{prefix}-{id:03} should be claimable"
        );
    }

    /// VT-1: the survey over a populated bare-remote reports holder + acquired per
    /// held id, and `--kind` filters by the id-space prefix segment (REQ-022, F-V7).
    #[test]
    fn vt1_survey_reports_holder_acquired_and_filters_by_kind() {
        let env = Substrate::new(2);
        hold(&env, 0, "SL", 148); // Agent 0
        hold(&env, 1, "SL", 7); // Agent 1
        hold(&env, 0, "IMP", 12); // Agent 0, a different kind

        // A fresh clone surveys from cold (it must fetch, then read).
        let surveyor = env.clone(0);

        // Unfiltered: all three held ids, holder + acquired populated, kinds mixed.
        let all = survey(surveyor, env.remote(), None).expect("survey all");
        let mut canon: Vec<&str> = all.iter().map(|h| h.canonical.as_str()).collect();
        canon.sort_unstable();
        assert_eq!(canon, vec!["IMP-012", "SL-007", "SL-148"]);
        for h in &all {
            assert!(!h.holder.is_empty(), "holder populated for {}", h.canonical);
            assert!(
                !h.acquired.is_empty(),
                "acquired populated for {}",
                h.canonical
            );
        }
        // Holder is the declaring agent's identity (set explicitly on the commit).
        let sl148 = all.iter().find(|h| h.canonical == "SL-148").unwrap();
        assert_eq!(sl148.holder, "Agent 0");
        let sl7 = all.iter().find(|h| h.canonical == "SL-007").unwrap();
        assert_eq!(sl7.holder, "Agent 1");

        // --kind = SL narrows to the SL id-space (not the IMP claim).
        let sl_only = survey(surveyor, env.remote(), Some("SL")).expect("survey SL");
        let mut sl_canon: Vec<&str> = sl_only.iter().map(|h| h.canonical.as_str()).collect();
        sl_canon.sort_unstable();
        assert_eq!(sl_canon, vec!["SL-007", "SL-148"]);
    }

    /// VT-2: a malformed / out-of-band ref under `refs/doctrine/reservation/*` is
    /// SKIPPED without aborting the listing (E3).
    #[test]
    fn vt2_malformed_ref_is_skipped_not_fatal() {
        let env = Substrate::new(1);
        hold(&env, 0, "SL", 1); // one well-formed claim

        // Plant out-of-band refs directly on the remote under the namespace:
        //  - a non-numeric id segment (`…/SL/main`)
        //  - a bare ref with no <prefix>/<NNN> tail (`…/garbage`)
        // Reuse the existing reservation ref's oid (the bare remote has no HEAD).
        let oid = git::git_text(
            &env.remote_path,
            &["rev-parse", "refs/doctrine/reservation/SL/001"],
        )
        .expect("rev-parse reservation oid");
        git_ok(
            &env.remote_path,
            &["update-ref", "refs/doctrine/reservation/SL/main", &oid],
        );
        git_ok(
            &env.remote_path,
            &["update-ref", "refs/doctrine/reservation/garbage", &oid],
        );

        // The survey still lists the well-formed claim; the malformed refs vanish.
        let held = survey(env.clone(0), env.remote(), None).expect("survey skips malformed");
        let canon: Vec<&str> = held.iter().map(|h| h.canonical.as_str()).collect();
        assert_eq!(
            canon,
            vec!["SL-001"],
            "only the well-formed ref survives (E3)"
        );
    }

    /// The canonical/holder/acquired derivation is exercised at the unit level too,
    /// independent of any remote — including the upper-casing and the E3 skips.
    #[test]
    fn parse_held_claim_derives_canonical_and_skips_malformed() {
        let row = |refname: &str| crate::git::RefRow {
            refname: refname.to_owned(),
            oid: "deadbeef".to_owned(),
            author: "Agent 9".to_owned(),
            date: "2026-01-01T00:00:00+00:00".to_owned(),
            msg: String::new(),
        };
        let ok = parse_held_claim(&row("refs/doctrine/reservation/sl/148")).expect("well-formed");
        assert_eq!(ok.canonical, "SL-148"); // <prefix> upper-cased, id zero-padded
        assert_eq!(ok.holder, "Agent 9");
        assert_eq!(ok.acquired, "2026-01-01T00:00:00+00:00");

        // E3 skips: non-numeric id, missing prefix, deeper sub-path, foreign namespace.
        for bad in [
            "refs/doctrine/reservation/SL/main",
            "refs/doctrine/reservation/garbage",
            "refs/doctrine/reservation/a/b/001",
            "refs/heads/main",
        ] {
            assert!(parse_held_claim(&row(bad)).is_none(), "skips {bad}");
        }
    }
}
