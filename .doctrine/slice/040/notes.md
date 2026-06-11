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
