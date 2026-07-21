# Design SL-212: Ingest hand-resolved trunk merge

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8). -->

Delivers **IMP-127** under **ADR-012 D4** as amended by **REV-030**: a sanctioned
verb that adopts an operator's hand-resolved `(base, source)` 3-way merge as a
conflicted candidate's `merge_oid`, validated by *provenance and content* (not
authorship), so `admit → integrate` proceed on genuine provenance with
publication staying fast-forward-only. Governance is settled (REV-030 applied);
this is a candidate-*construction* extension, not a trunk-publication change.

## 1. Design Problem

A conflicted `dispatch candidate create --worktree` dead-ends. Create runs an
object-db 3-way (`git merge-tree`); on conflict it parks the candidate branch at
`base_oid` with row `{status: Conflicted, merge_oid: ""}` and materialises a
worktree, but there is **no verb to feed a hand resolution back in**: `admit`
refuses (empty `merge_oid`), re-running `create` recomputes the same conflict.
The only escape (direct-land, SL-104) forfeits the admitted-OID CAS provenance
the candidate seam exists to give.

Design a verb — `candidate ingest` — that adopts the operator's resolved merge
commit as the candidate's `merge_oid`, **gated so a careless or arbitrary tree
cannot reach trunk**, and make `create` stage the conflict — from the *same*
merge engine ingest validates against — so the operator's natural `git commit`
yields a validatable merge.

## 2. Current State

- `candidate_create` (`src/dispatch.rs:1300`): `git merge-tree --write-tree`
  decides `Clean{tree}` → auto-commit `merge_oid`, or `Conflict`. On
  `Conflict + --worktree` (`:1409`) it parks the branch at `base_oid`, records
  `merge_oid=""`, status `Conflicted`, worktree checked out **clean** at base —
  no merge is staged, so the operator has no conflict to resolve.
- `candidate_admit` (`:1560`): validates `parents(merge_oid) == {base,source}`
  (order-**independent** `BTreeSet`, `:1620`) + `is-ancestor`; **no tree
  inspection**. Sufficient for a Doctrine-authored tree, insufficient for an
  operator-authored one (REV-030 F-1).
- `integrate`: FF-only on the admitted OID (`:2622`). **Unchanged by this slice.**
- `CandidateRow` (`src/ledger.rs:170`): "every field but `status` is immutable …
  never an in-place OID rewrite." `merge_oid` is `""` for a conflicted row.
- `git::merge_tree` (`src/git.rs:854`): returns `Clean{tree}` or a bare
  `Conflict` — **discards** the conflicted tree OID + stage table `merge-tree`
  emits on exit 1 (verified: the tree *is* written; the `:848` doc comment is
  wrong and is corrected here).
- `git::run_git`/runner (`src/git.rs:448`): neutralises only
  `core.autocrlf`/`core.eol`/`core.fileMode` — **not** merge strategy/config.
- `diff_doctrine_paths` (`src/git.rs:1053`): `git diff --name-only` **without**
  `--no-renames` (rename-folds) and UTF-8-lossy.
- `CandidateCommand` guard (`src/commands/guard.rs:315`): an **exhaustive** match
  (Create/Status/Admit); create/admit `Orchestrator`, status `Read`.
- `ledger::store` (`src/ledger.rs:384`): a **non-atomic** truncating
  `std::fs::write` (clippy-excepted "runtime coordination manifest"); the
  candidate ledger is a coordination-root **file**, not a committed ref.

## 3. Forces & Constraints

- **Governance (ADR-012 D4, REV-030).** Provenance validated, not authorship;
  binds "the resolved tree to the mechanical merge on non-conflicting **paths**;
  operator freedom … only at conflict loci, never an arbitrary tree"
  (`adr-012.md:122`); ordered parents (first parent `base_oid`); FF-only
  publication.
- **One merge engine — literally.** The reference merge must be *reproducible*
  from the recorded `(base_oid, source_oid)` at validation time. That reference
  is `git merge-tree` (ort, object-db). **`git merge` (worktree) is not
  equivalent** — per-branch/repo config (`branch.<n>.mergeOptions`,
  `merge.conflictStyle`, strategy) perturbs it but not `merge-tree` (verified:
  `mergeOptions=-Xours` → `git merge` exit 0, `merge-tree` exit 1 on identical
  inputs). So staging must **materialise merge-tree's own output**, never invoke
  `git merge`.
- **DRY / ADR-001.** Ride the git-diff seam; no parallel merge implementation.
  Materialising a *pre-computed* tree+index is materialisation, not merging.
  Pure/imperative split (AGENTS.md): validation pure; git reads in the shell.
- **Fail-closed.** Ambiguous ledger, criss-cross history, a custom merge driver,
  a non-merge tip, or a non-UTF-8-representable path must **refuse**, never guess.
- **Crash-durable pre-state, honestly bounded.** The ledger write is made atomic
  (temp+rename) so a crash cannot corrupt the manifest; full crash≡resume
  (restage after partial staging) is a *referenced follow-up*, not solved here.
- **Trust model.** The operator is the trusted human orchestrator; the content
  check is defence against *error*, not a sandbox against a malicious principal
  (who could direct-land). The security boundary is FF-only + admit + audit.

## 4. Guiding Principles

Provenance-and-content, not authorship. **One ort engine (merge-tree),
materialised — never a second `git merge` invocation.** Reproducible validation
from recorded OIDs. Fail-closed on ambiguity. FF-only publication untouched. As
simple as possible, but no simpler.

## 5. Proposed Design

### 5.1 System Model

```
 create (conflict+--worktree)              operator              ingest
 ─────────────────────────────────         ────────              ─────────────────────────────
 merge-tree(base,source) → T_c, C          resolve the           coord-root guard (refuse cand-worktree)
 guard: single base; no custom driver      markered files        select the one Conflicted row (write-once gate)
 CAS branch @ base_oid                      in the worktree,      recompute merge-tree(base,source) → T_c, C
 write Conflicted row (atomic, durable)     `git commit`          D = changed_paths(R^tree, T_c)  [--no-renames]
 MATERIALISE T_c into the worktree:           → R, 2-parent       validate: (i) parents==[base,source]
   read-tree T_c → checkout-index              [base, source]                (ii) D ⊆ C  (byte-wise)
   rewrite C entries to stages 1/2/3                                         (iii) markers at C  (advisory)
   set MERGE_HEAD=source                                          write-once fill: merge_oid=R, status=Created,
 (no `git merge` — config cannot perturb)                          ingested_at, merge_provenance=OperatorIngest
                                                        admit (unchanged) → integrate (FF-only, unchanged)
```

Create-staging and ingest-validation are the **same merge-tree output** by
construction — there is no `git merge` invocation for config to perturb (D2).

### 5.2 Interfaces & Contracts

**CLI.** `dispatch candidate ingest --slice <N> --label <L>` — no `--base`/
`--source` (from the recorded row). Run from the **coordination tree**; a
candidate-worktree cwd is refused (§5.4).

```rust
pub(crate) struct IngestRequest { pub slice: u32, pub label: String, pub ingested_at: String }
```

**Path identity is bytes.** Git paths are not UTF-8-guaranteed and `-z` output is
raw bytes; all path sets are byte paths so `D ⊆ C` is true tree inequality, not a
lossy string compare (F8).

```rust
pub(crate) struct IngestReject { pub reason: String }

pub(crate) fn validate_ingest_provenance(
    parents: &[String],                    // R's parents, in order
    base_oid: &str, source_oid: &str,
    diff_from_mechanical: &BTreeSet<Vec<u8>>, // D = changed_paths(R^tree, T_c), --no-renames
    conflict_paths: &BTreeSet<Vec<u8>>,       // C (non-empty; caller bails otherwise)
    marker_paths: &[Vec<u8>],                 // advisory: C-subset with surviving markers
) -> Result<(), IngestReject>;
//  (i)  parents == [base_oid, source_oid]      // ordered; covers single/reversed
//  (ii) diff_from_mechanical ⊆ conflict_paths  // "never an arbitrary tree" (byte-wise)
//  (iii) marker_paths.is_empty()               // ADVISORY (fails open on unreadable) — not a hard invariant
```

**Git helpers** (`src/git.rs`):

```rust
pub(crate) enum MergeTree {
    Clean { tree: String },
    Conflict { tree: String, stages: Vec<ConflictStage> },   // was: bare Conflict
}
pub(crate) struct ConflictStage { mode: String, oid: String, stage: u8, path: Vec<u8> }
// parse `merge-tree --write-tree -z --merge-base=<mb> ours theirs`: field 1 = tree oid;
// the stage table `<mode> <oid> <stage>\t<path>` → stages (already update-index --index-info shape).
// C = distinct stage paths.

fn changed_paths(root, tree_a, tree_b) -> Result<BTreeSet<Vec<u8>>>;
    // `git diff-tree --no-renames -r -z --name-only a b` — byte-safe, rename-fold OFF (soundness).
    // Generalise the diff_doctrine_paths seam (git.rs:1053) — one byte-safe primitive, thin wrappers.

fn merge_base_all(root, a, b) -> Result<Vec<String>>;   // `merge-base --all`; >1 ⇒ refuse (F5)
fn custom_merge_driver_paths(root, tree) -> Result<Vec<Vec<u8>>>;  // gitattributes non-built-in driver ⇒ refuse
```

`admit`/`integrate` are **unchanged**; `admit`'s set-check passes on ingest's
ordered parents; the ordered + content gate lives in `ingest`.

### 5.3 Data, State & Ownership

`CandidateRow` gains two backward-compatible fields:

```rust
#[serde(default)] pub ingested_at: String,                 // "" for non-ingested rows
#[serde(default)] pub merge_provenance: MergeProvenance;    // default Doctrine
#[derive(Default)] enum MergeProvenance { #[default] Doctrine, OperatorIngest }
```

**EX-3 refinement (write-once `merge_oid`), enforced.** The `ledger.rs:170`
contract is refined: `merge_oid` is **write-once** — settable exactly once on the
`Conflicted → Created` ingest transition (`""` → resolved OID); immutable
otherwise. **Enforcement is the fail-closed pre-state check** (§5.4 step 2:
select the *one* row with `status==Conflicted ∧ merge_oid==""`) — once `Created`,
no second ingest can rewrite it. Rationale (§7 D5): ingest **completes the same
candidate** (same base/source/`target_ref`) — it is not supersession.
`created_at`/`created_by` honestly stay create-time; `ingested_at` +
`merge_provenance=OperatorIngest` record *when* / *whose* authorship — RFC-016
§D's "recorded at the moment it happens."

**Atomic ledger write (Fork B).** `ledger::store` (`:384`) writes via
temp-file + `rename` (atomic on one filesystem) so a crash mid-write cannot
corrupt `candidates.toml` (or any dispatch manifest — a shared win). Backs the
durability claim (§3). Behaviour-preserving: readers see identical final content.

**Guard.** `CandidateCommand::Ingest { .. } => Orchestrator("dispatch-candidate-ingest")`
in the exhaustive match (`guard.rs:315`) + write-class golden (`src/main.rs`).
Candidate writes are **orchestrator-sole-writer** (worker-mode refused), so no
concurrent-writer protocol beyond atomic-store + the pre-state gate.

**Coordination-root identity (F10).** Ingest **refuses a candidate-worktree
cwd** — detected by `git rev-parse --git-common-dir != --git-dir` (a linked
worktree) *and/or* a resolved root under `.doctrine/state/dispatch/candidate/` —
with a message directing the operator to the coordination tree, and resolves the
ledger at the coordination root (never the candidate checkout's stale tree).

### 5.4 Lifecycle, Operations & Dynamics

**Create — conflict + `--worktree` arm (revised; other arms untouched):**

1. `merge-tree(base, source)` decides `Conflict`; retain `T_c` + stage table.
   **Guards:** `merge-base --all` == 1 (else refuse — criss-cross); **no
   custom (non-built-in) merge driver** on any merged path (else refuse —
   nondeterministic `C`; built-in `union`/`binary` allowed).
2. CAS-create the branch at `base_oid`.
3. **Write the `Conflicted` row (`merge_oid=""`) now** — atomic, durable, *before*
   the worktree (§3 / R-4).
4. `add_candidate_worktree` + `run_provision`.
5. **Materialise merge-tree's output into the worktree — no `git merge`** (D2):
   `git read-tree <T_c>` → `git checkout-index -af` (working files = `T_c`:
   markers at conflicts, merged elsewhere); for each path ∈ C rewrite its index
   entry to unmerged stages 1/2/3 via `git update-index --index-info` fed the
   stage lines (remove stage-0 first); write `MERGE_HEAD=source_oid`
   (+ `MERGE_MODE`/`MERGE_MSG`) so `git commit` yields a 2-parent
   `[base_oid, source_oid]` merge. Exact plumbing = `/plan` (OQ-1).
6. On any failure in 4/5: **roll back — remove worktree, delete row, delete ref
   (CAS `base_oid`, valid — nothing moved it)** — bail as an operational error.
7. stderr: "resolve the conflicts and `git commit`, then `candidate ingest` from
   the coordination tree."

**Operator:** resolve the materialised markers, `git commit` → `R`, a 2-parent
merge `[base_oid, source_oid]` on `target_ref`.

**Ingest:**

1. Resolve the coordination root; **refuse a candidate-worktree cwd** (§5.3).
2. `read_candidates`; select the **exactly-one** row for `label` with
   `status==Conflicted ∧ merge_oid==""` — refuse on zero/many/wrong pre-state
   (F12; this is the write-once gate).
3. `R = resolve_commit(target_ref)`; refuse `R == base_oid` (not committed).
4. **Guards:** single merge-base; no custom driver (as create).
5. `merge_tree(mb, base, source)` → `Conflict{T_c, C}`. `Clean` / empty `C` on
   exit-1 ⇒ **bail** (recorded conflict no longer reproduces — corruption).
6. `D = changed_paths(R^tree, T_c)` (`--no-renames -r -z`, byte paths); advisory
   marker scan of `R`'s blobs at `C` (attribute-aware `conflict-marker-size`,
   text-only via NUL detection, **fails open** on unreadable).
7. `validate_ingest_provenance(parents(R), base, source, D, C, marker_paths)` →
   `Ok` | refuse (reversed/single parent; altered non-conflict path `<p>`;
   surviving markers at `<p>`).
8. Re-read `target_ref == R` (best-effort drift — parity with `admit`; §7 D6).
9. **Write-once fill (atomic):** `merge_oid=R`, `status=Created`,
   `ingested_at`, `merge_provenance=OperatorIngest`. stdout `R`.

`admit → integrate` then proceed by the existing FF-only contract.

### 5.5 Invariants, Assumptions & Edge Cases

- **The predicate is one invariant:** `diff(R.tree, T_c) ⊆ C` (byte-wise,
  rename-fold OFF) — every path where `R` differs from git's mechanical merge is
  a conflict locus. Bracketed by ordered parents (i); markers (iii) advisory.
- **Rename detection OFF is load-bearing (F-new).** With default rename-folding a
  deleted non-conflict path can be hidden as a `x→c` rename; `--no-renames` makes
  `D` true pathwise inequality.
- **Path granularity (D3).** Freedom is per *conflicted path* (ADR-012
  "non-conflicting **paths**"); at a conflicted path the operator may rewrite
  content/mode/type/delete — a legitimate resolution. Residual (§8 R-1): a
  trusted operator may over-edit clean regions *within* a conflicted file;
  bounded by trust + FF-only + audit. Hunk-level is stricter than governance —
  out of scope.
- **Custom merge driver ⇒ refuse** (both create & ingest): nondeterministic /
  config-perturbed `C` breaks reproducibility. Built-ins deterministic, allowed.
- **Rename/rename & directory-rename** conflicts: resolving to a *third* name is
  ∉ C ⇒ refusal; v1 documented limitation (resolve to a recorded path, or
  supersede). No special code — the refusal teaches it; a test asserts it.
- **Binary conflicts:** marker scan skips (NUL-detected); the path ∈ C is already
  operator-free — harmless.
- **Empty `C` with exit-1 / non-UTF-8-only path in C:** defensive bail/refuse.
- **Assumption:** `merge-tree` (ort, single base, built-in drivers) is
  deterministic ⇒ create-materialised `T_c/C` == ingest-recomputed `T_c/C`.

## 6. Open Questions & Unknowns

- **OQ-1 — projection plumbing (plan detail).** Exact `read-tree` /
  `update-index --index-info` / `MERGE_HEAD`+`MERGE_MSG` sequence and its linked-
  worktree git-dir paths; a `ScratchIndex`-style throwaway `GIT_INDEX_FILE` may
  be cleaner. Resolved in `/plan`; no design fork remains.
- **OQ-2 — custom-driver detection surface.** Whether to inspect gitattributes
  per merged path or per changed path; lean per merged path (superset,
  fail-closed).

## 7. Decisions, Rationale & Alternatives

- **D1 — predicate = `diff(R.tree, T_c) ⊆ C`** (byte-wise, `--no-renames`). Rides
  git's ort merge; *is* ADR-012's path-level binding. Alt B (reclassify ourselves)
  reimplements merge — rejected. Alt C (parent+ancestry only) — rejected by
  REV-030 F-1.
- **D2 — create materialises merge-tree's output into the worktree; ingest
  recomputes merge-tree.** *Reversal:* an earlier draft staged via
  `git merge --no-commit`; codex pass 2 + a verified probe showed
  `branch.<n>.mergeOptions` makes `git merge ≠ git merge-tree`, so that staging
  was **not** the same engine. Literal materialisation (one engine) is the sound
  floor; it also dissolves the auto-commit-orphan (F13) and exit-code-ambiguity
  (F14) failure modes (no merge invocation). Alt: freeze all merge config via
  `-c` overrides — rejected (whack-a-mole; future knobs).
- **D3 — path-level conflict freedom** (governance-conformant; residual §8 R-1).
- **D4 — IMP-303 is a *related* follow-up, not a gate** (SL-212 ships at the
  clean-merge bar — REV-030's "no weaker" bound; ADR-012 `:266` makes no
  exact-OID-audit claim). `related` link recorded.
- **D5 — F16 via enforced write-once, not supersession.** Ingest completes the
  *same* candidate; supersession denotes a *different* one (ref proliferation,
  admit ambiguity, misrepresented lifecycle). Enforced by the fail-closed
  pre-state gate (§5.3).
- **D6 — TOCTOU re-read is best-effort drift** (parity with `admit:1637`); atomic-
  store + sole-writer close the corruption sub-concern; no lock protocol.
- **D7 — atomic `ledger::store`** (temp+rename): backs durability; shared win.
- **D8 — custom merge drivers refused; built-ins allowed** (reproducibility).
- **D9 — byte-path types** end-to-end (`-z` is bytes; non-UTF-8 safe).

**Refuted / down-scoped (evidence).** F2 `git merge`-vs-`merge-tree` divergence:
its *fix* (materialise, not merge) adopted — D2. F16 concurrency ("two
orchestrators"): outside the sole-writer model + parity with `admit`; atomic-store
handles crash-corruption. F1 path-vs-hunk: ADR-012 binds at path granularity
(D3). F3 audit-binding: ADR-012 makes no such claim (D4). F6 TOCTOU (D6). F13
conceded closed by codex.

## 8. Risks & Mitigations

- **R-1 — over-edit within a conflicted file.** Within governance (path-level);
  bounded by trust + FF-only + admit + audit. Disclosed, accepted.
- **R-2 — IMP-303 gap (inspectable ≠ inspected).** Pre-existing, authorship-blind;
  `related` link; ships at the clean-merge bar (D4).
- **R-3 — custom merge-driver divergence.** Refused at create & ingest (D8);
  built-ins verified equivalent.
- **R-4 — crash mid-staging.** Atomic-store (no manifest corruption) + durable
  row before the worktree. Residual (orphan ref on crash between branch and row —
  **pre-existing in `create` today**; full restage) → follow-up **IMP-305**,
  ref PRD-015 crash≡resume. Not absorbed here.
- **R-5 — non-UTF-8 path.** Byte-path types (D9); a path that still cannot be
  represented ⇒ fail-closed refuse.

## 9. Quality Engineering & Validation

**Pure unit tests** (`validate_ingest_provenance`, byte paths, no git): reversed
parents → reject; single parent → reject; `D ⊄ C` (arbitrary-tree) → reject;
marker present → reject (advisory); happy → accept.

**Integration (git fixtures):**
- happy: conflicted create leaves a materialised conflict (markers + unmerged
  index + `MERGE_HEAD==source`) → resolve → commit → ingest → `Created` → admit →
  integrate FF.
- **engine immunity:** set `branch.<n>.mergeOptions=-Xours` (and a custom driver)
  in the fixture → create still materialises merge-tree's `T_c` (not the config-
  perturbed merge) → ingest consistent. *This is the regression test for D2.*
- **rename-fold soundness:** operator deletes a non-conflict path folded as a
  rename → ingest **rejects** (proves `--no-renames`).
- refuse: reversed/single parent; arbitrary-tree (mutate a non-conflict path);
  surviving markers; non-`Conflicted`/ambiguous row; `R==base`; multiple merge
  bases; custom driver; candidate-worktree cwd.
- taxonomy: `conflict-marker-size`; binary; rename/rename, rename/delete,
  modify/delete, add/add, file/dir; mode/delete/symlink/gitlink; empty-C /
  malformed `-z`; non-UTF-8 / newline path names (byte-path round-trip).
- crash/atomicity: interrupted `store` leaves the prior manifest intact (temp+
  rename); rollback leaves no row/ref/tree.

**Behaviour surface (F18):** CLI help (`dispatch.rs:260`); `candidate status`
gains a conflicted→ingest prescription (`:1815`); admit provenance docs/errors
(`:1551/:1610`) and ledger field/status docs (`ledger.rs:144/:194`) drop
"Doctrine-created" absolutism; worker-guard golden; serialization goldens (new
`CandidateRow` fields, default-compat). **Behaviour-preservation:** clean +
non-worktree create/admit/integrate suites green unchanged; only the
conflict+worktree arm's worktree-state assertion changes.

**Governance verification (REV-030 payload → VT):** ADR-012 §Verification
operator-ingest case (`:266`) realised: arbitrary tree rejected; reversed parents
rejected; genuine 3-way accepted; FF-integrate by the same contract.

## 10. Review Notes

**Two adversarial passes — codex (GPT-5.5), workspace-read** — dispositioned on
evidence (§7, §8):

- **Pass 1 (7 blockers):** accepted F10/F11/F12/F5/F7/F8/F13/F14/F15/F16/F17/F18;
  refuted F1 (path-vs-hunk = governance), F2-example (git-2.54 probe), F3 (no
  audit-binding claim), F6 (TOCTOU parity).
- **Pass 2:** forced **D2 reversal** (materialise, not `git merge` — verified
  `mergeOptions` divergence); NEW rename-fold blocker (`--no-renames`); F8 byte
  paths; F10 concretised; crash/atomicity → atomic-store (D7) + IMP-305
  follow-up; custom-driver → refuse (D8); write-once enforcement made explicit.

**Pending — third pass (fresh agent):** targeted at the projection mechanism
(§5.4 step 5), the `--no-renames`/byte-path fixes, and the atomic-store, before
lock. See the handover.
