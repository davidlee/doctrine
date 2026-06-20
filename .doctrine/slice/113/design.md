# Design SL-113: Shared entity mutation seam over atomic write

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The entity engine (`entity.rs`) owns entity **creation** (`materialise*`,
`write_fileset`) and **listing** (`scan_ids`/`scan_named`), composing the leaf IO
seam `fsutil` (`write_atomic`, `safe_join`, `create_new_file`). It owns no
**mutation** path. Every in-place authored-file update is hand-rolled per kind:
read TOML → `toml_edit` splice → `std::fs::write(path, string)`. That byte-write
bypasses `fsutil::write_atomic`, which exists precisely to make authored writes
atomic (temp-write + rename).

Two costs: (1) 23 authored call sites re-spell the same non-atomic write — a
parallel-implementation smell against "no parallel implementation"; (2)
`std::fs::write` is non-atomic at the swap level — an interrupted userspace write
leaves a truncated/half-written committed `*.toml` that a concurrent reader or
the next command parses as corrupt, the failure `write_atomic` was built to
prevent.

**Scope of the guarantee (precise).** `write_atomic` is `write(tmp)` + `rename`
— it delivers **swap-atomicity** (a reader sees old-or-new, never torn; no
half-written authored file survives an interrupted userspace write). It is **not**
power-loss durability: there is no `fsync` of the temp or the parent dir, so a
kernel/power crash may still lose the most recent write (the *old* file remains
intact — never torn). Durability is out of scope (D4); the cost being closed is
reader-visible tearing + write duplication, not crash-durability.

## 2. Current State

Authored mutation, every kind, is `read_to_string` → `parse::<DocumentMut>()`
(or a pure helper) → mutate → `std::fs::write`. `write_atomic` is used by only 6
files; the authored *update* paths are not among them.

Two partial convergences already exist and matter:

- **`dep_seq.rs` holds shared authored write-cores.** `append` (`:178`), `remove`
  (`:246`), `set_authored_status` (`:356`), `append_string_array` (`:378`). Each
  ends in `std::fs::write`. `set_authored_status` is the status-flip seam for
  slice, requirement, backlog, revision, governance, knowledge/ASM — one write
  site behind seven kinds.
- **`relation.rs`** has the cross-kind edge writer (`append_edge`/`remove_edge`),
  also ending in `std::fs::write`.

The remainder are per-kind/per-command bespoke writes (memory, concept-map,
spec, integrity renumber, requirement `set_kind`, supersede, the map-server
concept-map route).

## 3. Forces & Constraints

- **ADR-001 (layering).** `fsutil` is the leaf IO seam; `write_atomic` is its
  "accountable, atomic" write. `entity` (engine) → `fsutil` (leaf) is the
  existing downward edge. Callers (command/engine) → `fsutil` is downward and
  legal. No new module dependency, no cycle.
- **Behaviour-preservation gate (AGENTS.md).** This touches shared machinery
  (`dep_seq` cores, `relation`, and the `write_atomic` seam itself under D4). The
  existing suites are the proof and must stay **green unchanged** — including
  `write_atomic`'s single-writer test, which D4 must not perturb (VT-3 *adds* the
  concurrency case rather than editing the existing one). Migration test edits
  held to **one failure-induction fixture** (reconciled SL-113 RV-113 F-1): the
  read→mutate logic carries no edits, but `spec.rs`'s orphan-on-append-failure
  test had to re-induce its forced write failure via a read-only **directory**
  (`0o555`) — `write_atomic` renames over a read-only target *file* where bare
  `fs::write` failed `EACCES` (rename keys on dir perm; §5.5 E3). The behavioural
  assertion is preserved; only the induction mechanism changed.
- **Storage rule.** In scope = **authored** `*.toml`/`*.md` only. Runtime state
  (`state.rs` phase sheets, `ledger.rs` manifest, `worktree.rs` marker) and
  derived/install (`install.rs`, `skills.rs`) deliberately stay on `fs::write`.
- **Memory `mem...unified-read-seam-does-not-deliver-a-unified-write-seam`.** A
  shared writer needs only a shared write *primitive*, not a shared on-disk shape
  or shared mutation logic. SL-113 shares exactly the byte-write; the per-kind
  `toml_edit` splice stays bespoke and untouched. This is the writer-*safety*
  gain (atomic) the memory names, nothing wider.
- **`as simple as possible`.** The seam already exists. The hypothesised
  `entity::save_meta` is rejected (D1).

## 4. Guiding Principles

- Reuse the existing seam; add no abstraction (D1).
- Migrate the byte-write only; leave read→mutate logic byte-identical.
- Make the authored/runtime boundary machine-enforced and self-documenting (D3).
- Maximum leverage: migrating the `dep_seq`/`relation` shared cores atomicizes
  many kinds through one site each.

## 5. Proposed Design

### 5.1 System Model

No new module, no new mutation abstraction. SL-113 has three moves:
1. **Migrate** the 23 authored byte-writes onto the existing
   `fsutil::write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()>`.
2. **Harden** `write_atomic`'s temp-naming so concurrent same-process writers to
   one path do not collide (D4) — a contained change to the leaf seam itself.
3. **Guard** with a `clippy` `disallowed-methods` rule making the migration
   permanent.

### 5.2 Interfaces & Contracts

`write_atomic`'s signature and return type are unchanged. Its contract: write a
sibling temp in the same directory, `rename` over the target (atomic on one
filesystem); a concurrent reader sees old-or-new, never torn. Callers keep their
existing signatures, return types, and error mapping.

**Temp-naming hardening (D4).** Today the temp is `.{name}.{pid}.tmp`
(`fsutil.rs:58`) — the pid disambiguates *processes* but **not** concurrent
*threads/tasks* of one process writing the same path (the map-server, axum/tokio,
is the one such writer). Add a process-global monotonic counter so every
`write_atomic` call gets a distinct temp:

```rust
static TEMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
// …
let seq = TEMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
let tmp = dir.join(format!(".{}.{}.{}.tmp", name.to_string_lossy(), std::process::id(), seq));
```

A counter (process-local mutable state, legal in the impure leaf seam) is chosen
over rng to keep the seam free of an rng dependency. Distinct temps mean two
concurrent same-path writers each rename their own bytes; the *last* rename wins
(last-writer-wins, as with any concurrent overwrite), and neither observes a torn
file nor a vanished-temp `rename` error. Cross-process disambiguation (the
existing pid term) is retained.

**Mechanical transform, every authored site:**

```
std::fs::write(p, doc.to_string())            →  fsutil::write_atomic(p, doc.to_string().as_bytes())
std::fs::write(p, next /* String */)          →  fsutil::write_atomic(p, next.as_bytes())
```

Surrounding error handling is preserved verbatim: `.with_context(|| …)` and
`.map_err(|e| DomainError…)` both compose over `write_atomic`'s `anyhow::Result`.
Add `use crate::fsutil;` where absent.

### 5.3 Data, State & Ownership

The 23 authored call sites (12 files):

| File | Sites | Role |
|---|---|---|
| `dep_seq.rs` | 178, 246, 356, 378 | shared cores: append / remove / `set_authored_status` (7 kinds) / `append_string_array` |
| `facet_write.rs` | 153 | `edit_in_place` — shared read→mutate→write-back core (oracle-found 2026-06-20; missed by the at-design sweep, caught by the §5.4 guard probe) |
| `relation.rs` | 784, 803 | `append_edge` / `remove_edge` |
| `memory.rs` | 2514, 2618, 2876 | memory edits |
| `concept_map.rs` | 1491, 1506, 1556, 1651 | `set_dsl` writes |
| `main.rs` | 4795, 4856, 4858 | supersede |
| `requirement.rs` | 396 | `set_kind` (`set_status` funnels through `dep_seq`) |
| `spec.rs` | 799 | member append |
| `integrity.rs` | 483 | renumber `id` |
| `backlog.rs` | 1816 | (status funnels through `dep_seq`) |
| `revision.rs` | 888 | (status funnels through `dep_seq`) |
| `map_server/routes.rs` | 412 | concept-map via HTTP |

**Out of scope** (stay on `fs::write`, get `#[expect]` — §5.4): `state.rs:409`
(runtime phase sheet), `ledger.rs:408` (runtime manifest), `worktree.rs:1862`
(runtime marker), `install.rs:586` + `skills.rs:637` (derived asset unpack),
`corpus.rs:403`/`406` (`sync_corpus` — shipped-corpus install, derived).

Line numbers are an at-design snapshot; the plan re-locates by function before
editing. **The authored set above is the *known* starting set, not a proven-
exhaustive hand count** — the `clippy` guard (§5.4) is the oracle: adding it and
running `just gate` surfaces every remaining production `fs::write`, each then
triaged by the rule *authored → migrate, runtime/derived → `#[expect]` + reason*.
`corpus.rs` was itself a hand-count miss (caught by the external pass, §10), and
`facet_write.rs:153` was a second miss (caught by the `/plan`-review oracle probe,
2026-06-20) — which is exactly why the guard, not the table, is authoritative.

### 5.4 Lifecycle, Operations & Dynamics

**Closure guard.** Add to `clippy.toml` `disallowed-methods`:

```toml
{ path = "std::fs::write", reason = "authored entity writes must route through fsutil::write_atomic (SL-113); runtime/derived sites carry an explicit #[expect]" }
```

`just gate` runs clippy bins/lib only (no `--all-targets`), so test code is
unlinted — no test-site noise. Deliberate non-authored production sites each
carry `#[expect(clippy::disallowed_methods, reason="…")]`, which documents the
authored/runtime boundary in code:

| Site | reason |
|---|---|
| `fsutil.rs:63` | the seam itself — `write_atomic`'s internal temp write |
| `state.rs:409` | runtime phase sheet — disposable, atomicity not required |
| `ledger.rs:408` | runtime coordination manifest |
| `worktree.rs:1862` | runtime worker marker |
| `install.rs:586` | derived asset unpack |
| `skills.rs:637` | derived asset unpack |
| `corpus.rs:403`, `:406` | derived — shipped-corpus sync into the items tree |

This inventory is the *known* exclusion set; the gate confirms completeness
(§5.3). A future authored mutation reaching for `fs::write` then fails the gate.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (behaviour preservation).** Each migrated site is reached on exactly
  the same branches as before; no-op-write guards (`if !changed { return }`,
  status no-op, relation `Noop`/`Absent`) are untouched, so mtime-hold semantics
  hold. `write_atomic` is invoked only where `fs::write` was.
- **INV-2 (swap-atomicity, scoped).** Every authored write is now temp+rename, so
  no reader ever sees a torn file and no interrupted *userspace* write leaves a
  half-written authored `*.toml`. This is **not** power-loss durability — no
  `fsync` (D4). Owned by `write_atomic`'s own test.
- **INV-3 (concurrent same-process writers).** After D4, two threads/tasks writing
  the same path use distinct temps; outcome is last-writer-wins with no torn file
  and no spurious `rename`-ENOENT. Pre-D4 the shared `.{name}.{pid}.tmp` could make
  one racer clobber/lose the other's temp — the gap the map-server route (the only
  multi-task writer) would otherwise inherit.
- **E1 (borrow lifetime).** `write_atomic(p, doc.to_string().as_bytes())` — the
  `String` temporary lives to end of statement; the `as_bytes()` borrow is valid.
  No allocation-count change vs `fs::write` (which also materialised the String).
- **E2 (`catalog/test_helpers.rs:16`) — CLOSED.** The module header declares
  "Compiles only under `#[cfg(test)]`"; `catalog/mod.rs` pulls it in for tests.
  The gate (no `--all-targets`) does not lint it → no `#[expect]` needed.
- **E3 (metadata not preserved — immaterial).** `rename`-replace yields a new
  inode; the target's mode/ACL/xattrs/hardlinks are not carried over, and the new
  file's mode is the temp's (`0666 & ~umask` → `0644`). Immaterial here: authored
  doctrine files are git-tracked `0644` TOML/MD with no special mode, ACL, or
  hardlinks, so `0644 → 0644` is a no-op; and `write_atomic` *already*
  rename-replaces authored files today (`memory.rs:1759/1797/2273`, `review.rs:1598`,
  `coverage_store.rs:75`) with no test observing inode/mode. A read-only target
  becoming replaceable via a writable dir is not a regression — doctrine never
  chmods authored files read-only.

## 6. Open Questions & Unknowns

- **OQ-1 (phase split).** One phase, or shared-cores (`dep_seq`/`relation`)
  before per-kind/command sites? Work is mechanical and behaviour-gated; lean
  **one phase**. Deferred to `/plan`.

## 7. Decisions, Rationale & Alternatives

- **D1 — the seam is the existing `fsutil::write_atomic`; add no new function.**
  Call-site reality: every site holds a fully-joined absolute path and a `String`
  body; the only shared thing is the byte-write, which `write_atomic` already is.
  *Rejected — (b) a thin `entity::save_authored` wrapper:* adds an indirection
  with no new capability (mutation logic stays per-kind, primitive stays in
  `fsutil`) — a miniature of the smell being closed. *Rejected — (c) a
  `(tree_root, rel)` containment-enforcing seam:* forces callers to thread a tree
  root they mostly don't hold; containment was already enforced at create time
  (no unsafe authored paths found). Cost without matching benefit.
- **D2 — authored-only scope.** Runtime/derived writes stay on `fs::write`.
  Matches the slice non-goal; gives a crisp closure invariant ("no *authored*
  update path calls `fs::write`"). `state.rs` phase sheets are `rm -rf`-able by
  design — runtime atomicity is a possible follow-up, not this slice.
- **D3 — closure via `clippy` `disallowed-methods` + explicit `#[expect]`.** The
  test-noise objection to a global ban does not apply (the gate skips tests). The
  annotations convert the authored/runtime boundary from tribal knowledge into
  documented-in-code intent. *Rejected — audit-grep only:* no permanent regression
  guard. **Reconciled SL-113 RV-113 F-3 — `#[expect]`, not `#[allow]`:** the repo
  enforces `Cargo.toml [lints] allow_attributes = "deny"`, so a bare `#[allow]`
  does not compile (this design originally mandated `#[allow]` and rejected
  `#[expect]`; enforced canon outranks that decision). The original objection to
  `#[expect]` — that it would fire `unfulfilled_lint_expectations` before the guard
  lands — is moot: the guard and the annotations land in the **same commit**
  (PHASE-03), so each `#[expect]` is fulfilled the moment it exists. A future stale
  `#[expect]` self-flags — a feature, and the tradeoff the repo already accepted
  globally.
- **D4 — harden `write_atomic` temp uniqueness; do not add `fsync`.** The pid-only
  temp name does not isolate concurrent same-process writers (the map-server is
  the one such caller). Migrating its route from raw `fs::write` (concurrent →
  torn bytes) to `write_atomic` is already strictly safer, but the shared temp
  would let one racer's `rename` consume the other's bytes. A process-global
  `AtomicU64` counter in the temp name closes it for all callers, contained to the
  leaf seam (§5.2). The existing `write_atomic` test stays green **unchanged**
  (temp still consumed by rename) — consistent with the behaviour-preservation
  gate — plus a new concurrency test (VT-3). *`fsync` rejected:* durability was
  never the cost in scope; adding it would change the seam's performance profile
  for all 6+ existing callers and is a separate, measured decision. The scoped
  claim is swap-atomicity (INV-2), stated as such.

## 8. Risks & Mitigations

- **R1 — context-frame drift.** Keeping `.with_context` nests `write_atomic`'s
  internal "Failed to rename" frame under the caller's "Failed to write {path}".
  Cosmetic; mitigate by confirming no test asserts on these strings at plan time.
- **R2 — map-server error mapping.** `routes.rs:412` maps to
  `MapServerError::ConceptMapIoError(e.to_string())`; `write_atomic`'s
  `anyhow::Error` stringifies cleanly — preserved.
- **R3 — missed authored site.** The slice scope itself undercounted (missed
  supersede + map-server), and a hand-count of production `fs::write` is **not
  reliable** — `corpus.rs:403/406` slipped a line-boundary sweep and was caught
  only by the external pass (§10). Mitigate **structurally**: the `clippy` guard is
  the oracle — adding it makes any remaining authored `fs::write` fail
  `just gate`, so a missed site breaks the build rather than silently shipping.
  The §5.3 table + §5.4 exclusions are the known starting set; execution reconciles
  against the gate, not against a claimed exhaustive count.
- **R4 — orphan temp in a committed tree — NOT A RISK.** A crash mid-write could
  leave `write_atomic`'s `.{name}.{pid}.{seq}.tmp` sibling in a committed authored
  dir. Two independent reasons it is moot: (a) `.gitignore` carries `*.tmp`, which
  matches the temp name; (b) `write_atomic` already writes committed authored
  trees today (`review.rs:1598`, `coverage_store.rs:75`, `memory.rs:1759/1797/
  2273`, `skills.rs:929`) — this slice introduces no new exposure.
- **R5 — supersede is per-file atomic, not cross-file transactional.**
  `run_supersede` writes NEW then OLD (`main.rs:4856/4858`). `write_atomic` makes
  each file's swap atomic but does **not** make the pair transactional — a crash
  between the two renames still leaves NEW-written/OLD-not. This is unchanged from
  the `fs::write` status quo (which was additionally torn-prone), so it is not a
  regression; INV-2's atomicity language is per-file, and cross-file
  transactionality is explicitly out of scope. Noted so the guarantee is not read
  wider than it is.

## 9. Quality Engineering & Validation

- **VT-1 — behaviour-preservation gate (primary).** Full suite green,
  **unchanged**. The per-kind suites (status round-trips, no-op-writes-nothing,
  malformed-refuse, relation append/remove idempotence, supersede, concept-map
  edits) prove the read→mutate logic is intact. No read→mutate test edits — that
  is the gate; the sole fixture change was one failure-induction mechanism
  (reconciled SL-113 RV-113 F-1; see §3).
- **VT-2 — `just gate` green** with the new disallowed-method and the `#[expect]`
  inventory. Proves no stray authored `fs::write` and that exclusions are
  explicit.
- **VA-1 — atomicity present.** `write_atomic` owns its swap test in `fsutil.rs`;
  per-site the property is "routes through the seam", verified by the guard
  (VT-2), not by added per-site fault injection (no new signal).
- **VT-3 — concurrent same-path writers (D4).** New `fsutil` test: N threads each
  `write_atomic` the same path concurrently; assert all return `Ok`, the final
  content equals one of the written payloads (never a mix), and no `.tmp` sibling
  remains. Knocks on the wall (drives the real seam, not a temp-name helper) and
  proves INV-3. The pre-existing `write_atomic_creates_then_overwrites_leaving_no_temp`
  test must remain green **unchanged** (behaviour-preservation of the single-writer
  contract under D4).

## 10. Review Notes

### Internal adversarial pass (2026-06-19)

- **Precedent confirms D1.** `write_atomic` is already the authored-mutation seam
  in `review.rs`, `coverage_store.rs`, `memory.rs`, `skills.rs`. `memory.rs` is
  **half-migrated** — atomic at `1759/1797/2273`, raw `fs::write` at
  `2514/2618/2876`. The inconsistency *within one file* is direct evidence the
  migration is right and the guard (D3) is what prevents the split recurring.
  The `doc.to_string().as_bytes()` transform (§5.2) is the established in-tree
  idiom (`memory.rs:2273`), not a new shape.
- **R4 (orphan temp) attacked and dismissed** — `*.tmp` gitignored + pre-existing
  usage (see §8 R4).
- **E2 (test-helper lint) attacked and closed** — `cfg(test)`-only, gate skips it
  (see §5.5 E2).
- **`#[expect]` inventory** — known starting set; the gate is the oracle (§5.3,
  §8 R3). *(The internal pass's "proven exhaustive by a 29-site sweep" claim was
  refuted by the external pass — see below.)*
- **Layering re-checked.** All callers (incl. `main`, `map_server/routes`) →
  `fsutil` is downward (command/engine → leaf); no new edge, no cycle (ADR-001).
- **Guard precision.** `disallowed-methods` on `std::fs::write` matches both
  `fs::write` and `std::fs::write` spellings (resolved-path match); it does **not**
  touch `File::write_all` (the `entity.rs` create path) — correct, that path is a
  separate seam and out of scope.

### External adversarial pass — codex / GPT-5.5 (2026-06-20)

Hostile review of the design + seam + call sites. Disposition:

- **[accepted → D4] Concurrent same-process writers share the temp.** Codex rated
  it a blocker on the in-scope map-server route. Downgraded (the migration is
  strictly safer than the raw-`fs::write` status quo it replaces — torn bytes vs a
  spurious rename error), but the gap is real in the existing seam. Closed by
  scoping in the temp-uniqueness hardening (D4, §5.2, INV-3, VT-3).
- **[accepted → R3/§5.3/§5.4] Inventory not hand-exhaustive.** Codex found
  `corpus.rs:403/406` (production `sync_corpus`) missing from both tables — a real
  miss from an interleaved-`cfg(test)` line-boundary sweep. Added to the exclusion
  set (derived) and reframed: the `clippy` guard, not a hand count, is the oracle.
  The internal pass's "29-site exhaustive" claim is retracted.
- **[accepted → §1/INV-2] Atomicity overstated.** No `fsync` ⇒ swap-atomicity,
  not power-loss durability. Scoped the claim throughout; `fsync` explicitly
  rejected as out of scope (D4).
- **[accepted → E3/R5] Metadata + multi-file.** Documented inode/mode
  non-preservation as immaterial for git-tracked `0644` authored files (E3, with
  precedent), and supersede's per-file (not cross-file) atomicity (R5).
- **[accepted → D1 framing] "Only centralizes one syscall."** Fair — D1 centralizes
  the byte-write *by design*; path construction / containment / multi-file
  orchestration stay per-kind (the `unified-read ≠ unified-write` memory: a shared
  writer needs a shared *primitive*, not a shared shape). The audit's "no mutation
  seam" is answered at the corruption + duplication layer, not as a grand unifier;
  broader unification would be the wrong seam.
- **[confirmed fine] Parent-dir absence, cross-fs rename, lint alias-matching /
  test-noise** — codex agreed these hold.
