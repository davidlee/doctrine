# SL-040 — design-exploration notes

Status: **exploration only** — design.md NOT yet authored, no decisions locked.
Captured for a fresh-context agent to run the full `/design` flow. Governing:
ADR-007 (D-C0…D-C11).

## Implementation surface map (from parallel investigators)

### Kind registration — mostly data-driven (`Kind` is data, not a trait)
The entity engine + governance spine are polymorphic over `&GovKind`; a new kind
gets `new`/`show`/`list`/`status` for free. **Bespoke per-kind work is small and
localized:**

- **new `src/review.rs`** — `REVIEW_KIND: GovKind` const (Kind, stem, statuses,
  hidden); `ReviewStatus` enum + `as_str()` + `REVIEW_STATUSES` const (canary test
  locks enum↔array); `render_review_toml`/`render_review_md` (splice user text via
  `toml_string()` — `mem.pattern.render.toml-splice-escape-user-values`); scaffold
  fn; thin `run_*` forwarders to the governance spine.
- **`src/integrity.rs:44–105`** — add a `KindRef` row to `KINDS` (the single
  corpus-wide id table). Miss it → kind escapes `validate`. `parse_canonical_ref`
  / `kind_by_prefix` read the table, no per-kind edit.
- **`src/main.rs`** — `mod review;`; `Command::Review` variant; `ReviewCommand`
  enum; arms in `conduct_on_command` (Write/Read classification) and `execute`.
  These are the **hardcoded match sites** (clap dispatch is not data-driven).
- **`install/manifest.toml` `[dirs].create`** + **`.gitignore` `!.doctrine/review/`**
  — `mem.pattern.install.authored-entity-wiring` (else silently uncommittable).
- **`install/templates/review.{toml,md}`** — compile-time embedded
  (`mem.pattern.build.rust-embed-no-rerun`: touch the embedding crate to re-embed).

### Verb family (exemplar: `memory`, the most verb-rich kind)
- CLI declare+dispatch: `src/main.rs:638–852` (MemoryCommand) + `1213–1312`
  (dispatch). Nested `Option<SubCommand>` pattern available.
- Handler/arg pattern: `memory.rs:744` `run_record(path, &RecordArgs)` — **bundle
  many flags into an arg struct** to dodge the clippy arg-ceiling
  (`mem.pattern.lint.cli-handler-args-struct`; trap at `main.rs:643–694`).
- **Edit-preserving status transition** (reuse, do not reinvent):
  `governance.rs:290–324` `set_status(g, root, id, status, today)` —
  `toml_edit::DocumentMut` preserves comments/unknown keys; no-op + malformed
  guards; date is shell-supplied (pure/imperative split). Used by `adr status`,
  `backlog edit`. **The dispose/verify/contest/withdraw transitions ride this.**
- Closed-enum + total fn (for D-C8): `adr.rs:44–63` (no-wildcard match),
  `backlog.rs:195–224` (`is_terminal()` predicate reused by hide-set). Canary test
  pattern keeps the const array in lockstep.

### Runtime state tree (baton / lock / warm-cache)
- `src/state.rs` — per-entity runtime subtree pattern is
  `.doctrine/state/<kind>/<NNN>/...` (slices use `.../slice/NNN/phases/`). Review
  mirrors as **`.doctrine/state/review/NNN/{baton,lock,cache}`**. Path computed
  from id, **never symlink-followed** (symlink is verified, not authority).
- `write_if_absent` (`state.rs:223`), edit-preserving `set_phase_status`
  (`state.rs:357`) are precedents.
- `.gitignore:33` already ignores all `.doctrine/state/` → review runtime state is
  covered, no new negation.

### Reusable primitives vs greenfield (ADR-007 D-C3/4/4a/10)
| Need | Status | Reuse |
|---|---|---|
| Atomic write | ✓ reuse | `fsutil.rs:50` `write_atomic(path, bytes)` (temp+rename) |
| Content hash | ✓ reuse | `git.rs:300` `sha256(bytes) -> String` (sha2 0.10) — the D-C10 staleness key |
| **File lock / CAS** | ✗ **greenfield** | no `flock`/lockfile anywhere; D-C4a is fresh |
| **Two-file authored+baton ordered write** | ✗ **greenfield** | engine does fileset atomicity (`entity.rs:440`), but D-C3 authored-first/baton-last temporal ordering is new |

### `reviews` edge + close-gate (D-C9b)
- Edge model: outbound-only (ADR-004), authored on the entity, harvested into the
  `Registry` (`registry.rs`) by `build_registry` (`spec.rs:823–904`). Review adds a
  `[target]` table (`ref` + optional `phase`) and a `reviews: Vec<ReviewEdge>`
  registry field + populator.
- **Reverse lookup: greenfield.** Registry has only forward checks. D-C9b needs a
  new `Registry::unresolved_blocker_reviews(subject_ref)` corpus scan (ADR-007
  already names this; not a new index).
- **Close-gate injection: `src/slice.rs:454–502` `set_slice_status`**, after the
  SeamBreach guard (~line 476), before the write path. Refuse terminal transition
  while any active RV targeting the subject has an unresolved `blocker`. `/close`
  skill already says this in prose — the gate operationalizes it.

## Open design threads (for /design — NOT resolved)

### R1 — warm-cache content-hash × worktree (largest unknown)
Worktree provisioning (`worktree.rs:71–80`) **withholds** `Tier::State`
(`.doctrine/state/**`), `Tier::Handover`, `Tier::Inquisition`, `Tier::MemoryCache`,
phase-links. **Consequence: a freshly-provisioned fork has NO `.doctrine/state/`**
→ the review baton AND warm-cache are **absent** in a fork (regenerable, by
design). Two implications for D-C10:
1. The cache is born cold in every fork — consistent with "regenerable runtime
   tier," fine.
2. The **explored path set** a review reads may include withheld/gitignored paths
   (`handover.md`, `inquisition.md`) that **exist in the main tree but not the
   fork** — so content-hashing them yields different results (or "missing") across
   trees. The staleness model must treat a path's *absence in this tree* as a
   defined state, not an error, and must not thrash when main-tree-only artefacts
   are unhashable in a fork. **Open: define the hash domain as "paths present in
   the current tree," and decide whether absence ⇒ stale or ⇒ excluded.**

### OQ-1 — `prime` runtime file shape
Mirror the phase pattern: `.doctrine/state/review/NNN/`. Open: one combined
`baton.toml` (await + rounds + cache pointer) vs separate `baton` / `cache` /
`lock` files. Lean: separate files — the lock must be a distinct OS-level artefact,
and the cache may be large; co-locating await+rounds in `baton.toml` is fine.

### OQ-2 — lock / CAS mechanism (greenfield)
No existing primitive. Options: (a) lockfile via `OpenOptions::create_new` as an
advisory mutex; (b) OS advisory `flock` (new dep); (c) optimistic CAS only — re-read
authored ledger after acquiring, compare a content-hash, abort on mismatch (D-C4a's
described shape). Lean: (a)+(c) — `create_new` lockfile for mutual exclusion +
sha256 CAS of the authored ledger for the lost-update guard. Reuses `sha256`.

## Forebrain threads (raised mid-exploration — design-shaping, see response)
These three questions bear directly on D-C10 and are captured as open threads:
- **T-a** — provenance of the domain_map: author-curated *relevant* set, NOT a
  mechanical *read* log as the staleness key. Git-changed-set seeds candidates;
  reviewer curates; content-hash protects the curated set. *Refinement (user):*
  read-hook instrumentation (harness-supported, not hard) could **seed/prompt
  attestation** — "you read these N paths, attest which are load-bearing" — raising
  domain_map accuracy without making the raw read-log the key. Seeding aid, not
  authority. Candidate future tooling idea.
- **T-b** — LLM token-cache warmth is unobservable/provider-dependent → doctrine
  makes NO attempt to detect it; the durable warm-cache is decoupled and works
  regardless. Naming collision: spec-driver's `warm` (= domain_map current vs
  drift) ≠ token-cache-warm — rename to avoid conflation (e.g. `current`).
- **T-c** — durable intra-file region reference: line ranges rot; doctrine lacks a
  durable sub-file anchor primitive. Lean: file-level content-hash as the staleness
  key (position-independent at file granularity); region anchors (symbol/heading)
  advisory-only. Possible backlog item: durable region-reference primitive.

---

## PHASE-02 implementation notes (authored kind end-to-end)

Durable carry-forwards for PHASE-03+ (the verb family rides this surface):

- **Eager-render seam (engine).** Review's fileset depends on facet/target/phase,
  which `entity::ScaffoldCtx` does not carry — the same reason memory has the
  `materialise_named` eager path. Added `entity::materialise_fresh_prebuilt` (the
  numbered twin: a `build(id, canonical) -> Fileset` closure under the shared
  claim-retry + H2 cleanup). `allocate_fresh` and it now share `claim_fresh_id`;
  the old `scaffold_and_write` was folded in and removed. `REVIEW_KIND.scaffold`
  is an inert stub (`review_scaffold_unused`) — review never rides `Kind.scaffold`.
- **D2 — scan-path id-only reader.** `meta::IdOnly`/`meta::read_id` deserialise
  `{ id }` ignoring the rest; `integrity::scan_kind` + `scan_aliases` use it. The
  shared strict `Meta` is UNCHANGED, so a corrupt status-bearing toml still hard-
  fails at `read_meta` (show/list/render). Leniency confined to the validate scan.
- **Authored schema is status-LESS** (D-C8): `review-NNN.toml` carries
  `id/slug/title` + `[review]` (facet/raiser/responder) + `[target]` (ref, optional
  phase) + append-only `[[finding]]`. Review's own readers (`ReviewDoc`/`FindingRow`
  in review.rs) parse it; derived status is computed at read time via PHASE-01's
  `derived_status`. NEVER ask the shared reader for a stored status.
- **Forward-edge validation** (`integrity::ensure_ref_resolves`, §7): `review new`
  refuses a dangling / unknown-prefix `[target].ref` BEFORE claiming an id (reuses
  `parse_canonical_ref` + a dir probe). `[target].phase` existence-check still
  deferred (minor, per design).
- **CLI surface** (main.rs hardcoded sites): `Command::Review` + `ReviewCommand`
  {New, List, Show}; conduct `New=Write`, List/Show=Read; `--facet` uses
  `review::Facet::parse` (the `MemoryType::parse` pattern, keeping the pure-core
  enum clap-free). `new` args bundled in `review::NewArgs` (arg-ceiling).
- **KINDS row**: `RV`, `stem="review"`, `state_dir=Some(".doctrine/state/review")`
  — the 2nd stateful kind (the baton/lock/cache tree lands in PHASE-03/05; the
  `.gitignore` `.doctrine/state/` already covers it, no new negation).
- **Install wiring**: manifest `[dirs].create += .doctrine/review`; repo `.gitignore`
  `!.doctrine/review/`; `install/templates/review.{toml,md}` embedded. The optional
  `[target].phase` line is injected by `render_review_toml` via a `{{target_phase}}`
  token (fixed-shape template, optional line rendered or empty).
- **Jail gotcha**: after editing `main.rs`/templates, the live binary can lag —
  `cargo build` may report `Finished 0.0s` while the on-disk bin is stale; `touch
  src/main.rs` (or verify with `cargo run -- review --help`) before trusting an
  e2e transcript. (mem.pattern.build.rust-embed-no-rerun-adjacent.)
- **For PHASE-03**: the `Verb`/`can()`/`render_finding`/`Finding`/`Severity` pure
  core is in place but still test-only (the module `expect(dead_code)` covers it);
  the verb handlers + `with_turn` baton/lock retire that suppression. Finding ids
  are `F-<max+1>` append-only over `ReviewDoc.finding`.

## PHASE-03 implementation notes (verb family + the turn guard)

Durable carry-forwards (the close-gate PHASE-04 + warm-cache PHASE-05 ride this):

- **`with_turn(root, id, verb, role, f)`** (review.rs) — the single coordination
  seam, the 8-step §6 protocol verbatim. `with_turn_hooked` is the same fn with an
  injectable `MidTurnHook` (a `&dyn Fn()` fired between the step-2 read and the
  step-5 write) — the deterministic test seam for the pre-write CAS window (no
  threads). Production `with_turn` passes a no-op hook.
- **Two CAS windows, distinct (D-C4a / Charge I).** ENTRY CAS (step 3):
  `sha256(authored) != baton.authored_hash` ⇒ heal the baton from authored truth
  (D-C2 recompute), bail. PRE-WRITE CAS (step 5): re-read bytes, `sha256 !=` the
  step-2 snapshot ⇒ bail before writing. A missing baton is COLD (proceed, the
  per-turn write seeds it). The lock serializes *invocations*; the CAS catches a
  *human* hand-edit the lock cannot see.
- **`LockGuard`** (RAII): `fsutil::create_new_file` lockfile at
  `.doctrine/state/review/NNN/lock` with a `pid`/`acquired` body; `Drop` removes it
  (normal + panic, NOT `-9`). `AlreadyExists` ⇒ "busy; re-run". `review unlock` is
  the hard-kill escape hatch (prints the body before removal). NOTE: `let _ignored =`
  in `Drop` (the must-use Result) — NOT `let _ =` (repo lint bans it).
- **Baton** (`baton.toml`, serde): `awaiting` (cached await display) ·
  `authored_hash` (CAS key) · `rounds` (bump/turn) · `contests` (counter) ·
  `handoff` (Vec<String>, the D10 ephemeral chatter; `--note` lands here, NEVER the
  ledger). `reconcile_baton_fields` is the recompute floor shared by entry-CAS heal,
  per-turn refresh, and `status`.
- **Edit-preserving finding edits** (governance.rs:290 at finding scope):
  `finding_table_mut(doc, id)` finds the `[[finding]]` by its `id` field (not array
  position), `apply_transition` sets status + optional responder pair via
  `toml_edit::value` (the structured-write twin of render's `toml_string`).
  `append_finding` pushes `F-<max+1>` (append-only). Comments / unknown keys / sister
  findings survive (VT-3 pins it).
- **Responsibility split (§6):** `with_turn` owns coordination + the STATIC
  verb→role check (`role != verb.required_role()`); the closure `f` owns the
  PER-FINDING `can(verb, Some(from), role)` gate (`gate()` helper) — only the verb
  knows its target finding id. `raise` targets `None` and is NOT await-blocked (D7).
- **Fork-root guard** lives in `resolve_review_root` (every verb routes through it):
  `worktree::is_linked_worktree(root)` true ⇒ bail (IMP-024). A non-git root returns
  Err ⇒ `.unwrap_or(false)` ⇒ treated not-a-fork (tempdir tests proceed).
- **Conduct (main.rs):** raise/dispose/verify/contest/withdraw/unlock = Write;
  list/show/**status** = Read (status mutates only the gitignored baton, never
  authored — Read-class per §9's prime/status rule). `--as` is `parse_role` (defaults
  to the verb's required role). Verb flags bundled in `RaiseArgs`/`DisposeArgs`
  (arg-ceiling).
- **VT-5 no-clobber simulation (the proof obligation, R-c):** (a) hold a `LockGuard`
  manually → second invocation bails busy, ledger untouched, loser re-runs post-drop
  from the refreshed baton; (b) mutate authored bytes directly (a crash-after-5,
  before-7) → next call's ENTRY CAS heals + bails, hash refreshed, no clobber; (c)
  `with_turn_hooked` injects a hand-edit mid-turn → PRE-WRITE CAS aborts, injected
  finding survives, F-1 not clobbered; (d) verify-then-contest same finding → the
  terminal-state per-finding gate refuses the loser. All assert FINAL on-disk ledger
  + baton, not just exit codes.

## PHASE-04 implementation notes (reverse close-gate + lifecycle teeth)

Durable carry-forwards (PHASE-06 /audit rewire rides the teeth):

- **`unresolved_blockers_for(root, subject_ref) -> Result<Vec<BlockerRef>>`**
  (review.rs, after `parse_ref`) — the standalone scoped scan (D8/D-C9b). Thin
  shell: `root.join(REVIEW_DIR)`, early-return empty if the tree is absent, then
  `read_reviews` (the existing PHASE-02 reader) and, per doc whose
  `target.reference == subject_ref`, the **pure** `doc_unresolved_blockers(doc)`.
  Pure check: `doc.derived().0 != Active ⇒ empty` (a Done ledger holds nothing,
  D-C9a), else findings with `Severity::parse == Blocker && !status.is_terminal()`
  → `BlockerRef { rv: canonical_id(doc.id), finding: f.id }` ("RV-007"/"F-2").
  NOT the spec `Registry`, NOT a reverse index. `review` does NOT import `slice`.
- **Close-shell injection** = `slice::run_status` (slice.rs ~388), the close
  COMMAND SHELL — NOT `set_slice_status`. Reads `from`, and iff
  `crosses_closure_seam(from, to)` calls `review::unresolved_blockers_for(&root,
  &canonical_id(id))`; non-empty ⇒ `bail!` listing `RV-NNN/F-n` BEFORE the FSM
  write (status untouched on refusal). The FSM writer stays focused; the one-way
  `slice-shell → review-query` coupling lives in the impure layer (ADR-001).
- **`crosses_closure_seam(from, to)`** (slice.rs, near `classify`) — true for
  EXACTLY `("audit","reconcile")` and `("reconcile","done")`. These are `Advance`
  in `classify`, so the gate canNOT key on `Transition::Advance` (other advances
  exist) — it keys on the explicit edge pair. Non-seam transitions (incl.
  `started→audit`) are never gated.
- **VT-5 sole-caller proof** (Charge VIII) = a SOURCE-grep test
  (`vt5_close_shell_is_the_sole_seam_crossing_caller_of_set_slice_status`):
  `include_str!("slice.rs")`, split at `#[cfg(test)]` to scope to PRODUCTION code,
  count `set_slice_status(` call sites excluding the `fn ` definition → assert
  exactly **1**. `set_slice_status` is module-private, so 1 production call site =
  one seam-crosser = the gate cannot be bypassed. A 2nd call site fails the test,
  forcing that caller to re-invoke the gate (or moving the gate to the FSM, per
  §7). **NO second seam-crossing caller exists today** — gate placement in the
  shell is sound.
- **VT coverage**: VT-1 (terminal⇒Done⇒not-reported; answered blocker keeps Active
  & gating) + VT-3 (scan correctness: match/ignore non-matching/non-blocker/
  terminal/no-tree) live in review.rs; VT-2 (refuse-then-pass via verify AND
  withdraw), VT-4 (seam-only firing + the predicate), VT-5 (sole-caller) live in
  slice.rs. Existing slice/close suites unchanged (behaviour-preservation gate
  held — no close test broke).
- **For PHASE-06**: the teeth are LIVE — `blocker`-severity findings on an Active
  RV targeting a slice refuse `slice status … reconcile|done`. The /audit rewire
  should lean on `blocker` as the sole gating severity (D-C9b); other severities
  record but never block close.

---

## PHASE-05 implementation notes (warm-cache + `prime`, D-C10 / §9)

Durable carry-forwards (PHASE-06 `/audit` rewire rides this surface):

- **`contentset` is now CONSUMED** — the module-level `cfg_attr(not(test),
  expect(dead_code))` is gone (its self-clearing interval ended). Two API methods
  had no production consumer (`ContentSet::is_stale_against`, the private
  `SetDrift::is_empty` it wraps) — the warm-cache takes the `diff` path directly
  (it needs the drifted path *list*, not a bool). They keep a scoped
  `cfg_attr(not(test), expect(dead_code, reason="…IMP-025 primitive surface"))`
  (the leaf is an IMP-025 candidate primitive; the broader API is deliberate).
  Added two thin accessors: `from_hashes(BTreeMap)` / `hashes() -> &BTreeMap` —
  the `[hashes]` table ⇄ `ContentSet` baseline round-trip.
- **`cache.toml` shape (as implemented), beside `baton.toml`/`lock` in
  `.doctrine/state/review/NNN/`** (runtime, gitignored, regenerable):
  `[[area]]` (`name`/`purpose`/`paths`) = the curated load-bearing domain_map
  (T-a); `[[invariant]]`/`[[risk]]` = `{text}` notes; `[hashes]` = the
  `ContentSet` over `⋃ area.paths` (the staleness baseline). serde structs
  `Cache`/`CacheArea`/`CacheNote`; `serde(rename)` maps `areas→area`,
  `invariants→invariant`, `risks→risk`. `[hashes]` is ALWAYS recomputed from
  `⋃ paths` on prime — any value the supplier put there is ignored, so the
  baseline can never drift from the domain_map.
- **`prime` flow.** `run_prime` is Read-class for authored conduct (mutates no
  authored ledger) but **acquires the PHASE-03 `LockGuard`** around the
  `cache.toml` write — lock ONLY, no baton, no CAS (§9). Two modes:
  - `--seed`: `git status --porcelain --untracked-files=all` via `git::git_text`
    (the existing impure seam — no new helper), parse `line.get(2..).trim()` +
    `rsplit(" -> ")` for renames → emit candidates; writes NOTHING, takes no lock.
    A *starting point* for curation, not authority (T-a).
    NB the porcelain parse is `get(2..).trim_start`, NOT `get(3..)` — the latter
    truncated deleted-file paths under the ` D ` spacing variant.
  - populate: domain_map from `--from <file>` or stdin → `validate_domain_map`
    (≥1 area; each named; each ≥1 path; every path root-relative — no absolute /
    `..`) → `contentset::compute(root, ⋃ paths)` → `write_cache`.
- **Staleness via `review status`** (extends the PHASE-03 verb). After the baton
  rebuild, *if a cache is primed*, it computes `stored.baseline().diff(compute(
  root, ⋃ paths))` and prints `cache: current` or `cache: stale (p1, p2, …)`
  (changed ++ removed[absence⇒stale, R1] ++ added, sorted+deduped). An
  optimization SIGNAL, never a gate — it cannot fail a verb. Unprimed ⇒ no cache
  line at all.
- **Pure/imperative split.** The staleness DIFF is pure (`contentset::diff`);
  `compute` (disk+sha2) and the cache read/write are the shell. Single PARENT
  root throughout; `resolve_review_root` (the fork guard) gates prime too (a
  review verb).
- **VT coverage** (all in review.rs `tests`): `vt1_prime_persists_domain_map_and_
  hashes_then_current` (VT-1 — asserts `cache.toml` content + `[hashes]` ==
  `compute(⋃ paths)` + `Current`); `vt2_status_reports_current_then_stale_on_
  drift_and_absence` (VT-2 — current → mutate bytes ⇒ stale naming the path →
  remove ⇒ stale naming it, R1). Plus: prime ignores supplied `[hashes]`; refuses
  empty/no-path/absolute-path domain_maps (no write on refusal); serializes via
  the lock (held lock ⇒ "busy", no clobber); status silent when unprimed.
  Existing review/contentset suites unchanged (behaviour-preservation held).
- **NOT built (additive, deferred):** a `subject` root for pre-import fork review
  → IMP-024; region anchors (file-level hash is the key) → IDE-002; read-hook
  attestation seeding (T-a) → future tooling.
