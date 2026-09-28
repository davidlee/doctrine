// SPDX-License-Identifier: GPL-3.0-only
//! `reserve` — claim-backend selection for fresh-id allocation (SL-148).
//!
//! [`backend`] is the single seam that resolves which [`Claim`](crate::entity::Claim)
//! backend a Fresh-allocating materialise site uses, and the matching
//! scan source ([`ScanSource`]) the claim loop unions into its candidate set. It is
//! the SOLE selector (`LocalFs` / [`CloneRef`] / [`GitRef`]): it loads `[reservation]`,
//! performs the reachability fetch, and decides degradation per design D8 (§5.4). Routing the 11
//! Fresh call sites through one helper — rather than a literal `&LocalFs` at each — is
//! what lets the second backend drop in behind a single signature (design §5.2, F-3).
//!
//! Layering (ADR-001): `reserve` is engine. It reaches `entity` (engine, same tier)
//! and the leaf seams `git`/`dtoml`/`kinds`/`corpus_guard` (downward). The
//! interactive D8 y/N prompt is NOT imported (that would be an upward edge to
//! `install` = command); instead the prompt is injected as a `PromptFn` from the
//! command-tier caller (the pure/imperative split — the impurity is passed in).

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::corpus_guard::DOCTRINE_PATHSPEC;
use crate::entity::{Acquired, Claim, ClaimCtx, LocalFs};
use crate::git;
use crate::kinds::Kind;

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
/// The clone-local reservation namespace (SL-269): claims arbitrated by this clone's
/// own ref store when the remote is out of reach. Same `<prefix>/<NNN>` layout as
/// [`RESERVATION_REF_PREFIX`]; never fetched, never pushed.
const RESERVATION_LOCAL_REF_PREFIX: &str = "refs/doctrine/reservation-local";
/// The `[reservation]` config key opting into local fallback when the remote is
/// unreachable — named once for the declined error and the fallback prompt (STD-001).
/// The serde field keeps its own kebab-case rename.
const ALLOW_LOCAL_FALLBACK_KEY: &str = "allow-local-fallback";
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
            // Same-machine exclusion + keeps the loop's H2 cleanup valid (D1).
            git::RefCas::Updated => seat_claimed_dir(ctx.dir, &canonical, OnExisting::Occupied),
            // A rival created the ref first — lost the race; recompute and retry.
            git::RefCas::Moved { .. } => Ok(Acquired::AlreadyHeld),
        }
    }

    #[cfg(test)]
    fn arbiter(&self) -> crate::entity::Arbiter {
        crate::entity::Arbiter::RemoteRef
    }
}

// ---------------------------------------------------------------------------
// CloneRef backend (SL-269)
// ---------------------------------------------------------------------------

/// The local-reach backend inside a git repository: [`GitRef`] without the remote.
/// The claim linearizes on a zero-oid create CAS over
/// `refs/doctrine/reservation-local/{prefix}/{id:03}` in the clone's COMMON git dir,
/// so it arbitrates across every worktree of the clone (SL-269 design sec-2, DEC-337).
struct CloneRef {
    root: PathBuf,
    prefix: String,
    holder_name: String,
    holder_email: String,
}

impl CloneRef {
    /// The clone-local reservation ref for candidate `id`.
    fn refname(&self, id: u32) -> String {
        format!("{RESERVATION_LOCAL_REF_PREFIX}/{}/{id:03}", self.prefix)
    }
}

impl Claim for CloneRef {
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired> {
        let refname = self.refname(ctx.id);
        let canonical = format!("{}-{:03}", self.prefix, ctx.id);
        let new_oid = git::commit_empty_tree_as(
            &self.root,
            &canonical,
            &self.holder_name,
            &self.holder_email,
        )
        .with_context(|| format!("Failed to build reservation commit for {canonical}"))?;
        match git::update_ref_cas(&self.root, &refname, &new_oid, git::ZERO_OID)
            .with_context(|| format!("Failed to create reservation {refname}"))?
        {
            // An existing dir is a ref-less writer's claim (a pre-slice binary, or by
            // hand): the id is burnt, retry the next one.
            git::RefCas::Updated => seat_claimed_dir(ctx.dir, &canonical, OnExisting::AlreadyHeld),
            // Another tree of this clone created the ref first — retry.
            git::RefCas::Moved { .. } => Ok(Acquired::AlreadyHeld),
        }
    }

    #[cfg(test)]
    fn arbiter(&self) -> crate::entity::Arbiter {
        crate::entity::Arbiter::CloneRef
    }
}

/// What an already-existing dir means once the reservation CAS has been won — the
/// one per-backend difference in [`seat_claimed_dir`] (SL-269 design sec-2).
#[derive(Clone, Copy)]
enum OnExisting {
    /// The dir is a same-clone rival's claim: lost the race, recompute and retry.
    AlreadyHeld,
    /// The dir is foreign (the `GitRef` split state): hard error with the reseat hint.
    Occupied,
}

/// Seat the entity dir after the reservation CAS has been won — the ONE post-CAS
/// `mkdir` outcome mapping every backend shares (SL-269, RV-406 `F-3`), so the error
/// texts live here once. `Won` only when THIS call created `dir` (the claim loop owns
/// and cleans it up on a later failure). An existing path maps per `on_existing`; any
/// other io failure is a hard error that keeps the io cause and names the burnt id —
/// never mistaken for the split state (STD-003).
fn seat_claimed_dir(
    dir: &Path,
    canonical: &str,
    on_existing: OnExisting,
) -> anyhow::Result<Acquired> {
    match std::fs::create_dir(dir) {
        Ok(()) => Ok(Acquired::Won),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => match on_existing {
            OnExisting::AlreadyHeld => Ok(Acquired::AlreadyHeld),
            // E1 split-state (remote won, local dir already exists / foreign):
            // hard error with the reseat hint, never orphan silently (R3).
            OnExisting::Occupied => Err(anyhow::anyhow!(
                "reservation {canonical} pushed to the remote but its local dir \
                 {} could not be created (split state). Run `doctrine reseat {canonical}` \
                 and pick another id.",
                dir.display()
            )),
        },
        Err(e) => Err(e).with_context(|| {
            format!(
                "reservation {canonical} is held but its dir {} could not be created; \
                 the id is burnt, re-run to allocate the next one",
                dir.display()
            )
        }),
    }
}

/// The ONE scan source both git backends (`CloneRef`, `GitRef`) use (SL-269 design
/// sec-2): per call, `local` (this tree's dirs, own ids first) ∪ `siblings` (the
/// sibling worktrees' dirs, read once at construction) ∪ this kind's
/// `reservation-local` refs ∪ its `reservation` refs. With `fetch = Some(remote)` the
/// remote namespace is re-fetched first, so a rival's post-`AlreadyHeld` ref widens
/// this iteration's set. Both ref reads are scoped to `prefix` — pooling kinds drives
/// `next_id` to the global max+1 (ISS-221).
fn composed_scan_source(
    root: &Path,
    prefix: &str,
    siblings: Vec<u32>,
    fetch: Option<String>,
) -> ScanSource {
    let root = root.to_path_buf();
    let prefix = prefix.to_owned();
    Box::new(move |local: &[u32]| {
        if let Some(remote) = &fetch {
            git::fetch_refspec(&root, remote, RESERVATION_REFSPEC)
                .with_context(|| format!("Failed to fetch reservations from {remote}"))?;
        }
        let mut ids: Vec<u32> = local.to_vec();
        ids.extend_from_slice(&siblings);
        ids.extend(reservation_ids(
            &root,
            RESERVATION_LOCAL_REF_PREFIX,
            &prefix,
        )?);
        ids.extend(reservation_ids(&root, RESERVATION_REF_PREFIX, &prefix)?);
        Ok(ids)
    })
}

/// Where a doctrine project sits in its git worktree (SL-269): the worktree top
/// level, and the project root relative to it (`<rel>`, empty in the usual case).
struct GitLocus {
    toplevel: PathBuf,
    rel: PathBuf,
}

impl GitLocus {
    /// The doctrine project root inside worktree `tree` (`<tree>/<rel>`). An empty
    /// `<rel>` yields `tree` itself — `Path::join("")` would add a trailing separator
    /// that leaks into the skip warning.
    fn project_in(&self, tree: &Path) -> PathBuf {
        if self.rel.as_os_str().is_empty() {
            tree.to_path_buf()
        } else {
            tree.join(&self.rel)
        }
    }
}

/// The project's [`GitLocus`], or `None` when `root` is outside any git worktree —
/// the one git-vs-plain-dir decision for the local arms.
fn git_locus(root: &Path) -> anyhow::Result<Option<GitLocus>> {
    Ok(git::toplevel_and_prefix(root)
        .with_context(|| format!("Failed to locate {} in git", root.display()))?
        .map(|(toplevel, rel)| GitLocus { toplevel, rel }))
}

/// Sibling worktrees' ids for one kind, and the siblings that could not be read
/// (STD-003: "found nothing" and "could not read" stay distinguishable).
#[derive(Debug, Default)]
struct SiblingScan {
    ids: Vec<u32>,
    /// `(sibling project root, reason)` per sibling not scanned.
    skipped: Vec<(PathBuf, String)>,
}

/// Read `<sibling>/<rel>/<kind dir>/NNN` across the clone's live worktrees, the
/// invoking tree excluded (its dirs arrive through the scan's `local` ids). Live =
/// not bare, not prunable, path present (the `git::live_worktree_for_ref` rule). A
/// sibling with no doctrine root at `<rel>`, or whose kind dir cannot be read, goes
/// on `skipped`; one with a root but no kind dir contributes nothing. A failing
/// `git worktree list` is a hard error — it is not a per-sibling fault.
fn read_siblings(locus: &GitLocus, kind: &Kind) -> anyhow::Result<SiblingScan> {
    let records = git::list_worktrees(&locus.toplevel).with_context(|| {
        format!(
            "Failed to list the worktrees of {}",
            locus.toplevel.display()
        )
    })?;
    let own = std::fs::canonicalize(&locus.toplevel)
        .with_context(|| format!("Failed to resolve {}", locus.toplevel.display()))?;
    let mut scan = SiblingScan::default();
    for record in records.iter().filter(|r| !r.bare && !r.prunable) {
        let tree = match std::fs::canonicalize(&record.path) {
            Ok(tree) => tree,
            // A listed path that is gone is not a live tree.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                scan.skipped
                    .push((locus.project_in(&record.path), format!("{e}")));
                continue;
            }
        };
        if tree == own {
            continue;
        }
        let project = locus.project_in(&tree);
        if !project.join(DOCTRINE_PATHSPEC).is_dir() {
            let reason = format!("no doctrine root at {}", display_rel(&locus.rel));
            scan.skipped.push((project, reason));
            continue;
        }
        match crate::entity::scan_ids(&project.join(kind.dir)) {
            Ok(ids) => scan.ids.extend(ids),
            Err(e) => scan.skipped.push((project, format!("{e:#}"))),
        }
    }
    Ok(scan)
}

/// `<rel>` for a message: the worktree top level reads as `./`, not an empty string.
fn display_rel(rel: &Path) -> String {
    if rel.as_os_str().is_empty() {
        "./".to_owned()
    } else {
        rel.display().to_string()
    }
}

/// The stderr warning for a sibling worktree left out of the id scan (STD-003).
fn sibling_skip_warning(project: &Path, reason: &str) -> String {
    format!(
        "doctrine: worktree {} not scanned for reserved ids ({reason})",
        project.display()
    )
}

/// [`read_siblings`] with each skipped sibling warned on stderr (never stdout); the
/// allocation proceeds on the ids that could be read.
fn sibling_ids_warned(locus: &GitLocus, kind: &Kind) -> anyhow::Result<Vec<u32>> {
    use std::io::Write;
    let scan = read_siblings(locus, kind)?;
    for (project, reason) in &scan.skipped {
        drop(writeln!(
            std::io::stderr(),
            "{}",
            sibling_skip_warning(project, reason)
        ));
    }
    Ok(scan.ids)
}

/// The reserved ids in this clone's ref store under `namespace` FOR `prefix` — parse
/// the trailing `<NNN>` of every `<namespace>/<prefix>/<NNN>` (design §5.3). The
/// namespace is [`RESERVATION_REF_PREFIX`] (the fetched remote claims) or
/// [`RESERVATION_LOCAL_REF_PREFIX`] (the clone-local claims, SL-269). Scoped to
/// `<prefix>/` so a sibling kind's ids never leak into this kind's candidate set
/// (ISS-221) — the allocation twin of `survey`'s `held_prefix` filter. Unparseable
/// ref names under the namespace are ignored, not fatal (E3).
fn reservation_ids(root: &Path, namespace: &str, prefix: &str) -> anyhow::Result<Vec<u32>> {
    let rows = git::for_each_ref(root, &format!("{namespace}/{prefix}/"))
        .with_context(|| format!("Failed to enumerate reservation refs under {namespace}"))?;
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
/// for `kind` — its `prefix` keys the reservation ref segment (F-V7), its `dir` the
/// sibling-worktree scan (SL-269). Loads `[reservation]`, then delegates to
/// [`resolve_backend`] — the SOLE backend selector (design EX-3). `prompt` injects the D8 y/N
/// confirmation (the command-tier caller passes `install::prompt_confirm`).
pub(crate) fn backend(
    root: &Path,
    kind: &Kind,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    let cfg = load_reservation_config(root)?;
    // The ONE ambient-env read (ISS-483): selection below is a function of its inputs.
    resolve_backend(root, kind, &cfg, env_fallback_optin(), prompt)
}

/// The SOLE backend selector / reachability probe / degradation decider (design
/// EX-3, D8). The reachability fetch *is* the probe; its ids seed the `GitRef`
/// scan. "Local" below is [`local_backend`]: `CloneRef` in a git repo, else `LocalFs`
/// (SL-269). Degradation:
/// - `local` ⇒ local, the remote is never touched (EX-5).
/// - `shared` ⇒ `GitRef`; a fetch failure hard-errors, no fallback (shared is shared).
/// - `auto` + **no remote configured** ⇒ local + a one-time stderr signal (the
///   genuine single-tree fallback).
/// - `auto` + **configured remote that fails** ⇒ hard error by default; the operator
///   opts into local fallback per allocation via the env opt-in / config
///   `allow_local_fallback` / the interactive y/N `prompt` (TTY) — on accept ⇒
///   local + the one-time signal.
///
/// `fallback_optin` is the env opt-in, read once by [`backend`] and passed in — never
/// read here, so selection is hermetic by construction (ISS-483, SL-269).
fn resolve_backend(
    root: &Path,
    kind: &Kind,
    cfg: &ReservationConfig,
    fallback_optin: bool,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    match cfg.reach {
        Reach::Local => local_backend(root, kind),
        Reach::Shared => {
            let remote = require_remote(root, cfg, "shared")?;
            // Reachability probe (this fetch is also the GitRef scan's first fetch).
            probe_reachability(root, &remote).with_context(|| {
                format!("reach=shared: reservation remote {remote} unreachable")
            })?;
            gitref(root, kind, &remote)
        }
        Reach::Auto => resolve_auto(root, kind, cfg, fallback_optin, prompt),
    }
}

/// The `auto` degradation decision (D8).
fn resolve_auto(
    root: &Path,
    kind: &Kind,
    cfg: &ReservationConfig,
    fallback_optin: bool,
    prompt: PromptFn,
) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    let Some(remote) = configured_remote(root, cfg)? else {
        // Structurally single-tree: the genuine PRD-005 fallback case.
        signal_local_fallback("no remote configured");
        return local_backend(root, kind);
    };
    match probe_reachability(root, &remote) {
        Ok(()) => gitref(root, kind, &remote),
        Err(e) => {
            // Configured remote that FAILS: fail-closed unless the operator opts in.
            if fallback_optin || cfg.allow_local_fallback || prompt_fallback(&remote, prompt)? {
                signal_local_fallback(&format!("remote {remote} unreachable: {e}"));
                local_backend(root, kind)
            } else {
                Err(e).with_context(|| {
                    format!(
                        "reach=auto: reservation remote {remote} unreachable and local fallback \
                         declined. Set [reservation] {ALLOW_LOCAL_FALLBACK_KEY}=true or \
                         {ENV_FALLBACK}=1 to allocate locally."
                    )
                })
            }
        }
    }
}

/// Construct the `GitRef` backend + its re-fetching composed scan for `remote`.
/// Sibling worktrees are read here, once (design sec-2).
fn gitref(root: &Path, kind: &Kind, remote: &str) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    // A reachable remote implies a repo; a root outside one has no siblings.
    let siblings = match git_locus(root)? {
        Some(locus) => sibling_ids_warned(&locus, kind)?,
        None => Vec::new(),
    };
    let (holder_name, holder_email) = git::resolve_holder(root);
    let backend = GitRef {
        root: root.to_path_buf(),
        prefix: kind.prefix.to_owned(),
        remote: remote.to_owned(),
        holder_name,
        holder_email,
    };
    let scan = composed_scan_source(root, kind.prefix, siblings, Some(remote.to_owned()));
    Ok((Box::new(backend), scan))
}

/// The ONE local-reach backend, for every local arm (`reach = local`, `auto` with no
/// remote, `auto` with accepted fallback): `CloneRef` + the composed scan (no fetch)
/// when `root` is inside a git worktree, else `LocalFs` + the identity scan (SL-269
/// EX-1). Sibling worktrees are read here, once.
fn local_backend(root: &Path, kind: &Kind) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
    let Some(locus) = git_locus(root)? else {
        return Ok((Box::new(LocalFs), local_scan_source()));
    };
    let siblings = sibling_ids_warned(&locus, kind)?;
    let (holder_name, holder_email) = git::resolve_holder(root);
    let backend = CloneRef {
        root: root.to_path_buf(),
        prefix: kind.prefix.to_owned(),
        holder_name,
        holder_email,
    };
    Ok((
        Box::new(backend),
        composed_scan_source(root, kind.prefix, siblings, None),
    ))
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
        "{}",
        fallback_prompt_text(remote)
    ));
    prompt("")
}

/// The D8 fallback prompt (SL-269 design sec-2): says the id is scoped to this clone
/// and names both levers that skip the prompt, from their single-source constants.
fn fallback_prompt_text(remote: &str) -> String {
    format!(
        "reservation remote {remote} is unreachable. Allocate this id in this clone only? [y/N]\n\
         (to skip this prompt: [reservation] {ALLOW_LOCAL_FALLBACK_KEY} = true, or {ENV_FALLBACK}=1)\n"
    )
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
            RESERVATION_LOCAL_REF_PREFIX,
            "refs/doctrine/reservation-local"
        );
        // The remote namespace's scan root never string-prefixes the local one.
        assert!(!RESERVATION_LOCAL_REF_PREFIX.starts_with(&format!("{RESERVATION_REF_PREFIX}/")));
        assert_eq!(ALLOW_LOCAL_FALLBACK_KEY, "allow-local-fallback");
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

    use crate::entity::Arbiter;

    /// A never-y prompt: declines local fallback (the default D8 posture).
    fn decline(_p: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    /// Reach selection with the env opt-in passed in, never read (ISS-483): load
    /// `root`'s config and resolve for the `TK` id-space with a declining prompt.
    fn select(root: &Path, optin: bool) -> anyhow::Result<(Box<dyn Claim>, ScanSource)> {
        let cfg = load_reservation_config(root)?;
        resolve_backend(root, &TK, &cfg, optin, decline)
    }

    /// The test kind: id-space `TK`, tree `.doctrine/tk` under the project root.
    const TK: Kind = Kind {
        dir: ".doctrine/tk",
        prefix: "TK",
        stem: "tk",
    };

    /// A test kind for id-space `prefix` (the ref segment is all the remote tests read).
    fn kind(prefix: &'static str) -> Kind {
        Kind { prefix, ..TK }
    }

    /// ISS-281 guard: a test that needs a NON-git root asserts it rather than
    /// trusting `TMPDIR` to sit outside every repository.
    fn assert_outside_git(root: &Path) {
        assert!(
            git_locus(root).unwrap().is_none(),
            "precondition: {} must be outside any git worktree (ISS-281)",
            root.display()
        );
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
            let path = self.clone(i).join(crate::dtoml::DOCTRINE_TOML);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, body).unwrap();
        }
    }

    /// VT-1: collision-freedom under contention. Two clones compute the SAME
    /// candidate id; exactly one create-push lands; the loser re-fetches, recomputes,
    /// and lands the next id — no duplicate holder (I1, REQ-020/021).
    #[test]
    fn vt1_two_clones_racing_the_same_id_do_not_collide() {
        let env = Substrate::new(2);

        // Clone 0 reserves id 1 (the first candidate over an empty namespace).
        let (b0, _s0) = gitref(env.clone(0), &kind("TK"), env.remote()).unwrap();
        let dir0 = env.clone(0).join("tree/001");
        std::fs::create_dir_all(env.clone(0).join("tree")).unwrap();
        let won0 = b0.claim(&ClaimCtx { dir: &dir0, id: 1 }).unwrap();
        assert!(matches!(won0, Acquired::Won), "first clone wins id 1");

        // Clone 1 computes the same candidate (1) — its create-push must be rejected
        // (the ref already exists on the remote): a lost race, not a duplicate.
        let (b1, mut s1) = gitref(env.clone(1), &kind("TK"), env.remote()).unwrap();
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

    /// ISS-221 regression: the GitRef scan source is scoped to its OWN prefix. Two
    /// kinds sharing a remote must not pool ids — a fresh `RSK` allocation sees only
    /// `RSK` reservations, never the higher `SL` ids, so per-kind counters stay
    /// independent on the remote arm (they always were on `LocalFs`). The pre-fix
    /// scan enumerated the whole namespace and returned the union, making every
    /// kind's `next_id` the global max+1.
    #[test]
    fn gitref_scan_source_is_scoped_to_its_own_prefix() {
        let env = Substrate::new(1);
        // A high SL reservation and a low RSK reservation coexist on one remote.
        hold(&env, 0, "SL", 148);
        hold(&env, 0, "RSK", 2);

        // The RSK scan must return ONLY the RSK id (2), never the SL id (148).
        let (_b, mut scan) = gitref(env.clone(0), &kind("RSK"), env.remote()).unwrap();
        let ids = scan(&[]).unwrap();
        assert_eq!(ids, vec![2], "RSK scan is scoped to RSK reservations only");
        // ⇒ the next RSK id is 3, not 149 (no cross-kind pooling).
        assert_eq!(crate::entity::next_id(&ids, &[]), 3);
    }

    /// SL-269 VT-3: the ref reader is scoped to one `(namespace, prefix)` — the
    /// remote and clone-local namespaces never leak into each other, nor do kinds.
    #[test]
    fn reservation_ids_is_scoped_to_namespace_and_prefix() {
        let env = Substrate::new(1);
        let root = env.clone(0);
        for (ns, prefix, id) in [
            (RESERVATION_REF_PREFIX, "SL", "001"),
            (RESERVATION_REF_PREFIX, "ASM", "002"),
            (RESERVATION_LOCAL_REF_PREFIX, "SL", "003"),
            (RESERVATION_LOCAL_REF_PREFIX, "ASM", "004"),
        ] {
            git_ok(
                root,
                &["update-ref", &format!("{ns}/{prefix}/{id}"), "HEAD"],
            );
        }
        for (ns, prefix, want) in [
            (RESERVATION_REF_PREFIX, "SL", 1),
            (RESERVATION_REF_PREFIX, "ASM", 2),
            (RESERVATION_LOCAL_REF_PREFIX, "SL", 3),
            (RESERVATION_LOCAL_REF_PREFIX, "ASM", 4),
        ] {
            assert_eq!(
                reservation_ids(root, ns, prefix).expect("read ids"),
                vec![want],
                "{ns}/{prefix} yields exactly its own id"
            );
        }
    }

    /// VT-4 (e2e): the reservation commit's tree is the empty tree (no blobs); the
    /// entity record carries no coordination bytes (REQ-024, I2). The empty-tree
    /// content-freedom is asserted at the git layer; here we confirm the GitRef claim
    /// path produces it.
    #[test]
    fn vt4_gitref_claim_is_content_free() {
        let env = Substrate::new(1);
        let (b, _s) = gitref(env.clone(0), &kind("SL"), env.remote()).unwrap();
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
        assert_eq!(
            tree,
            git::empty_tree_oid(&env.remote_path).expect("derive empty tree")
        );
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
        let (b, _s) = select(root, false).expect("local backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::CloneRef,
            "local backend in a clone must be CloneRef (no remote contact)"
        );

        // shared with an unreachable remote: hard error (no fallback).
        env.write_config(
            0,
            "[reservation]\nreach = \"shared\"\nremote = \"/no/such/remote\"\n",
        );
        assert!(
            select(root, false).is_err(),
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
        let (b, _s) = select(root, false).expect("shared backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::RemoteRef,
            "shared + reachable remote selects GitRef"
        );

        // auto with a reachable remote: GitRef.
        env.write_config(
            0,
            &format!(
                "[reservation]\nreach = \"auto\"\nremote = \"{}\"\n",
                env.remote()
            ),
        );
        let (b, _s) = select(root, false).expect("auto backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::RemoteRef,
            "auto + reachable remote selects GitRef"
        );
    }

    /// VT-3 / EX-3: `auto` + **no remote configured** degrades to local (the genuine
    /// single-tree fallback); `auto` + a **configured remote that fails** hard-errors by
    /// default (D8 fail-closed) and accepts local fallback only on explicit opt-in.
    #[test]
    fn vt3_auto_degradation_is_fail_closed_with_explicit_optin() {
        let env = Substrate::new(1);
        let root = env.clone(0);

        // auto + no remote configured (and none in .git/config) ⇒ local (CloneRef).
        env.write_config(0, "[reservation]\nreach = \"auto\"\n");
        let (b, _s) = select(root, false).expect("auto no-remote backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::CloneRef,
            "auto + no remote ⇒ CloneRef in a clone"
        );

        // auto + a configured remote that FAILS, prompt declines ⇒ hard error.
        env.write_config(
            0,
            "[reservation]\nreach = \"auto\"\nremote = \"/no/such/remote\"\n",
        );
        assert!(
            select(root, false).is_err(),
            "auto + failing configured remote hard-errors when fallback declined"
        );

        // Same, but config opt-in (allow-local-fallback) ⇒ local (never silent).
        env.write_config(
            0,
            "[reservation]\nreach = \"auto\"\nremote = \"/no/such/remote\"\nallow-local-fallback = true\n",
        );
        let (b, _s) = select(root, false).expect("opt-in fallback backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::CloneRef,
            "explicit opt-in ⇒ local (CloneRef) fallback"
        );
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
        assert_outside_git(tmp.path());
        let cfg = ReservationConfig::default();
        let (b, _s) =
            resolve_backend(tmp.path(), &TK, &cfg, false, decline).expect("auto non-git ⇒ LocalFs");
        assert_eq!(
            b.arbiter(),
            Arbiter::Dir,
            "default auto in a non-git dir must degrade to LocalFs, not error"
        );
    }

    /// VT-5: E1 split-state (remote-won / local-mkdir-failed) hard-errors with the
    /// `doctrine reseat <canonical>` remediation (D6/R3) — no silent orphan.
    #[test]
    fn vt5_split_state_hard_errors_with_reseat_hint() {
        let env = Substrate::new(1);
        let (b, _s) = gitref(env.clone(0), &kind("SL"), env.remote()).unwrap();
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

    /// SL-269 VT-2 (RV-406 `F-3`): a post-CAS mkdir that fails for any reason other
    /// than the path already existing is a hard error that keeps the io cause and
    /// names the burnt id — never the split-state `reseat` remediation.
    #[test]
    fn post_cas_mkdir_io_error_keeps_cause_without_reseat_hint() {
        let env = Substrate::new(1);
        let (b, _s) = gitref(env.clone(0), &kind("SL"), env.remote()).unwrap();
        // No `tree/` parent: create_dir(tree/009) fails with NotFound after the win.
        let dir = env.clone(0).join("tree/009");
        let err = b.claim(&ClaimCtx { dir: &dir, id: 9 }).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("No such file"), "keeps the io cause: {msg}");
        assert!(msg.contains("SL-009"), "names the burnt id: {msg}");
        assert!(
            !msg.contains("reseat"),
            "no reseat hint off split-state: {msg}"
        );
    }

    /// VT-6 (back-compat seam): a `local` backend never contacts a remote, and with
    /// no refs and no siblings its scan returns the own dirs unchanged.
    #[test]
    fn vt6_local_backend_is_back_compatible() {
        let env = Substrate::new(1);
        env.write_config(0, ""); // no [reservation] table at all
        let (b, mut s) = select(env.clone(0), false).expect("default backend");
        assert_eq!(
            b.arbiter(),
            Arbiter::CloneRef,
            "no [reservation] in a clone ⇒ CloneRef (EX-5)"
        );
        // No refs, no siblings: the scan returns the own dirs, in order.
        assert_eq!(s(&[3, 7]).unwrap(), vec![3, 7]);
    }

    /// SL-269 VT-4 (ISS-483): selection is a function of its inputs — the env opt-in
    /// arrives as a `bool`, never read inside the selector, so the ambient
    /// `DOCTRINE_RESERVATION_FALLBACK` of the test process cannot flip the outcome.
    #[test]
    fn reach_selection_ignores_ambient_fallback_env() {
        let env = Substrate::new(1);
        let root = env.clone(0);
        env.write_config(
            0,
            "[reservation]\nreach = \"auto\"\nremote = \"/no/such/remote\"\n",
        );
        assert!(
            select(root, false).is_err(),
            "auto + unreachable remote, opt-in off ⇒ hard error"
        );
        let (b, _s) = select(root, true).expect("opt-in on ⇒ local fallback");
        assert_eq!(
            b.arbiter(),
            Arbiter::CloneRef,
            "opt-in on ⇒ local (CloneRef)"
        );
    }

    // -----------------------------------------------------------------------
    // SL-269 PHASE-02: clone-wide local reservation (CloneRef + composed scan),
    // over one clone with two linked worktrees and no remote.
    // -----------------------------------------------------------------------

    use crate::test_support::LinkedTrees;

    /// `reach = local`, the arm every clone-wide test selects through.
    fn local_cfg() -> ReservationConfig {
        ReservationConfig {
            reach: Reach::Local,
            ..ReservationConfig::default()
        }
    }

    /// Create `root`'s `TK` tree and return it.
    fn tk_tree(root: &Path) -> PathBuf {
        let tree = root.join(TK.dir);
        std::fs::create_dir_all(&tree).unwrap();
        tree
    }

    /// Claim `id` in `tree` through `backend`.
    fn claim_at(backend: &dyn Claim, tree: &Path, id: u32) -> anyhow::Result<Acquired> {
        backend.claim(&ClaimCtx {
            dir: &tree.join(format!("{id:03}")),
            id,
        })
    }

    /// VT-1: the `CloneRef` CAS in the common git dir arbitrates across worktrees —
    /// two trees computing the same candidate cannot both win it.
    #[test]
    fn two_linked_trees_allocating_one_kind_get_distinct_ids() {
        let lt = LinkedTrees::new("");
        let (a, b) = (lt.root(&lt.a), lt.root(&lt.b));
        let (ba, _sa) = resolve_backend(&a, &TK, &local_cfg(), false, decline).unwrap();
        let (bb, mut sb) = resolve_backend(&b, &TK, &local_cfg(), false, decline).unwrap();
        assert_eq!(ba.arbiter(), Arbiter::CloneRef);
        let (ta, tb) = (tk_tree(&a), tk_tree(&b));

        assert!(matches!(
            claim_at(ba.as_ref(), &ta, 1).unwrap(),
            Acquired::Won
        ));
        assert!(
            matches!(
                claim_at(bb.as_ref(), &tb, 1).unwrap(),
                Acquired::AlreadyHeld
            ),
            "tree b loses id 1 to tree a"
        );
        assert!(!tb.join("001").exists(), "the loser seats no dir");

        let next = crate::entity::next_id(&sb(&[]).unwrap(), &[]);
        assert_eq!(next, 2, "b's scan sees a's ref, so it recomputes to 2");
        assert!(matches!(
            claim_at(bb.as_ref(), &tb, 2).unwrap(),
            Acquired::Won
        ));

        let rows = git::for_each_ref(&lt.main, "refs/doctrine/reservation-local/TK/").unwrap();
        let ids: Vec<&str> = rows
            .iter()
            .filter_map(|r| r.refname.rsplit('/').next())
            .collect();
        assert_eq!(ids, vec!["001", "002"], "one clone-local ref per id");
    }

    /// VT-2: a sibling tree's uncommitted entity dir (a pre-slice or hand-made mint)
    /// is in the candidate set, so it is never reissued.
    #[test]
    fn scan_sees_a_sibling_trees_uncommitted_entity_dir() {
        let lt = LinkedTrees::new("");
        std::fs::create_dir_all(lt.root(&lt.a).join(TK.dir).join("005")).unwrap();
        let (_b, mut s) =
            resolve_backend(&lt.root(&lt.b), &TK, &local_cfg(), false, decline).unwrap();
        assert!(
            s(&[]).unwrap().contains(&5),
            "sibling a's TK-005 is scanned"
        );
    }

    /// Optional hardening: `<rel>` is honoured — the sibling's kind dir is read under
    /// its doctrine root below the worktree top level, not at the top level.
    #[test]
    fn scan_honours_the_doctrine_root_below_the_worktree_top() {
        let lt = LinkedTrees::new("proj/");
        std::fs::create_dir_all(lt.root(&lt.a).join(TK.dir).join("006")).unwrap();
        let b = lt.root(&lt.b);
        let locus = git_locus(&b).unwrap().expect("b is in git");
        let scan = read_siblings(&locus, &TK).unwrap();
        assert!(
            scan.skipped.is_empty(),
            "every tree has proj/.doctrine: {scan:?}"
        );
        let (_b, mut s) = resolve_backend(&b, &TK, &local_cfg(), false, decline).unwrap();
        assert!(
            s(&[]).unwrap().contains(&6),
            "a's proj/.doctrine/tk/006 is scanned"
        );
    }

    /// VT-3: the scan reads both ref namespaces, each scoped to the kind's prefix —
    /// a clone that switches reach stays collision-free, and ISS-221 does not regress.
    #[test]
    fn scan_sees_both_ref_namespaces_scoped_to_prefix() {
        let lt = LinkedTrees::new("");
        for refname in [
            format!("{RESERVATION_LOCAL_REF_PREFIX}/TK/004"),
            format!("{RESERVATION_REF_PREFIX}/TK/007"),
            format!("{RESERVATION_LOCAL_REF_PREFIX}/OT/050"),
        ] {
            git_ok(&lt.main, &["update-ref", &refname, "HEAD"]);
        }
        let (_b, mut s) =
            resolve_backend(&lt.root(&lt.a), &TK, &local_cfg(), false, decline).unwrap();
        let ids = s(&[]).unwrap();
        assert!(ids.contains(&4), "reservation-local/TK: {ids:?}");
        assert!(ids.contains(&7), "reservation/TK: {ids:?}");
        assert!(
            !ids.contains(&50),
            "another kind's ref never leaks: {ids:?}"
        );
    }

    /// VT-4: once the `CloneRef` CAS is won, an existing dir is a ref-less writer's
    /// claim — `AlreadyHeld` (retry the next id), the ref kept (burnt, not rolled
    /// back). Any other mkdir failure keeps its cause, names the id, and has no
    /// `reseat` hint (RV-406 `F-3`).
    #[test]
    fn clone_ref_existing_dir_is_already_held() {
        let lt = LinkedTrees::new("");
        let a = lt.root(&lt.a);
        let (b, _s) = resolve_backend(&a, &TK, &local_cfg(), false, decline).unwrap();
        let tree = tk_tree(&a);
        std::fs::create_dir(tree.join("003")).unwrap();
        assert!(matches!(
            claim_at(b.as_ref(), &tree, 3).unwrap(),
            Acquired::AlreadyHeld
        ));
        let held = git::for_each_ref(&a, "refs/doctrine/reservation-local/TK/003").unwrap();
        assert_eq!(held.len(), 1, "the ref stays held (burnt, not rolled back)");

        // No parent dir: the post-CAS mkdir fails with an io error, not AlreadyExists.
        let err = claim_at(b.as_ref(), &a.join("no-such-parent"), 4).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("No such file"), "keeps the io cause: {msg}");
        assert!(msg.contains("TK-004"), "names the burnt id: {msg}");
        assert!(!msg.contains("reseat"), "no reseat hint: {msg}");
    }

    /// VT-5: a sibling with no doctrine root at `<rel>` is not scanned, is named on
    /// the skipped list and in its warning, and the allocation still proceeds.
    #[test]
    fn sibling_without_doctrine_root_is_warned_not_scanned() {
        let lt = LinkedTrees::new("");
        std::fs::remove_dir_all(lt.b.join(DOCTRINE_PATHSPEC)).unwrap();
        let a = lt.root(&lt.a);

        let locus = git_locus(&a).unwrap().expect("a is in git");
        let scan = read_siblings(&locus, &TK).unwrap();
        let b_canon = std::fs::canonicalize(&lt.b).unwrap();
        assert_eq!(scan.skipped.len(), 1, "only b is skipped: {scan:?}");
        let (path, reason) = &scan.skipped[0];
        assert_eq!(path, &b_canon, "the skipped sibling is b");
        assert!(reason.contains("no doctrine root"), "{reason}");
        let warning = sibling_skip_warning(path, reason);
        assert!(
            warning.contains(&format!("worktree {} not scanned", b_canon.display())),
            "names b's path exactly, no trailing separator: {warning}"
        );
        assert!(warning.contains(reason), "{warning}");

        let (b, _s) = resolve_backend(&a, &TK, &local_cfg(), false, decline).unwrap();
        assert!(matches!(
            claim_at(b.as_ref(), &tk_tree(&a), 1).unwrap(),
            Acquired::Won
        ));
    }

    /// VT-6: `reach = local` is a plain `mkdir` outside git and the clone-wide ref CAS
    /// inside it.
    #[test]
    fn local_reach_is_clone_ref_in_git_and_dir_outside() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_outside_git(tmp.path());
        let (b, _s) = resolve_backend(tmp.path(), &TK, &local_cfg(), false, decline).unwrap();
        assert_eq!(b.arbiter(), Arbiter::Dir, "non-git root keeps LocalFs");

        let lt = LinkedTrees::new("");
        let (b, _s) = resolve_backend(&lt.root(&lt.a), &TK, &local_cfg(), false, decline).unwrap();
        assert_eq!(b.arbiter(), Arbiter::CloneRef, "git root takes CloneRef");
    }

    /// SL-269 EX-5: the fallback prompt says the id is clone-scoped and names both
    /// levers that skip it, from their single-source constants.
    #[test]
    fn fallback_prompt_names_scope_and_both_levers() {
        let text = fallback_prompt_text("origin");
        assert!(
            text.contains("reservation remote origin is unreachable"),
            "{text}"
        );
        assert!(text.contains("in this clone only"), "{text}");
        assert!(text.contains(ALLOW_LOCAL_FALLBACK_KEY), "{text}");
        assert!(text.contains(&format!("{ENV_FALLBACK}=1")), "{text}");
    }

    // -----------------------------------------------------------------------
    // PHASE-04: held-claims survey (doctrine reservation list, REQ-022).
    // -----------------------------------------------------------------------

    /// Reserve `id` under `prefix` from clone `c` so the survey has something to read.
    fn hold(env: &Substrate, c: usize, prefix: &'static str, id: u32) {
        let (b, _s) = gitref(env.clone(c), &kind(prefix), env.remote()).unwrap();
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
