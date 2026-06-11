# Design SL-040: RV review-ledger kind + review verb family

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-040, ADR-007, ADR-006, ADR-004, ADR-001, IMP-022, IMP-024, IMP-025,
     IDE-002); ADR-007 decisions cited D-C0…D-C11; design-local decisions bare
     D1…D11; doc-local refs bare — OQ-1 (§12), R1 (§13), VT/VA (§14). -->

## 1. Design Problem

Build the **RV review kind** end to end — the first-class adversarial-review
primitive ADR-007 (accepted) decides. One generic `facet`-parameterized ledger
reviews any subject via the outbound `reviews` edge, coordinated by a turn-based
baton in runtime state with CLI-mediated turn-taking and a per-review lock.
Realises IMP-001. This slice **builds the ADR**; it does not re-decide it —
tensions with ADR-007 go through `/consult`, not local drift.

Closure intent: the kind exists; all verbs work under the turn guard; the
concurrent-write test shows no clobber; status/close-gate behave per D-C8/D-C9;
the context cache primes and invalidates on content drift; and `/audit` produces
an RV instead of a hand-made `audit.md`.

## 2. Current State

- **No structured review primitive.** `audit.md` is hand-authored (the known
  scaffold gap), inquisition is informal, code-review unstructured.
- **Entity engine** (`src/entity.rs`): kind-blind scaffold engine; `Kind` is data
  (dir, prefix, scaffold fn), not a trait. New kinds get id-allocation + scaffold
  for free.
- **Two kind shapes today.** `adr` (`src/adr.rs`) is a thin `GovKind` forwarder —
  its status is *stored & hand-set*. `slice` (`src/slice.rs`) is a raw
  `entity::Kind` with bespoke commands, a derived rollup, and a `state_dir`.
- **`integrity::KINDS`** (`src/integrity.rs:44`) — the single corpus-wide id table;
  `scan_kind` reads `meta::read_meta(...).id` for every kind. `Meta.status`
  (`src/meta.rs:38`) is mandatory (no default).
- **Reusable primitives:** `fsutil::write_atomic` (`fsutil.rs:50`), `sha256`
  (`git.rs:300`), runtime-state helpers (`state.rs:223` `write_if_absent`,
  `:357` `set_phase_status`), edit-preserving status transition
  (`governance.rs:290`), the spec `Registry` (spec-validation-specific —
  `registry.rs`/`spec.rs:823`), the slice close FSM (`slice.rs:454`
  `set_slice_status`, closure seam at `:471`).
- **Greenfield:** file lock/CAS (no `flock`/lockfile anywhere); the
  authored-first/baton-last ordered two-file write; a reverse lookup over the
  outbound `reviews` edge (relations are outbound-only, ADR-004).

## 3. Forces & Constraints

- **ADR-007 is canon** (D-C0…D-C11, accepted). The design implements; it does not
  re-decide. Where this doc *interprets* an under-specified ADR clause (D2, D6,
  D7) it says so and stays inside ADR intent.
- **ADR-001 layering** — leaf ← engine ← command, no cycles.
- **ADR-006 shared boundary** (D-C7) — review is single-locus, turn-based; the
  dispatched-fork model is the worktree target (§4, D4).
- **Storage rule** — authored TOML carries no derived data; derived status and the
  baton/cache are runtime-tier, gitignored, regenerable.
- **No parallel implementation / DRY** — ride existing seams; the one genuinely
  reusable new primitive (`contentset`) is factored for lift, not generalised
  speculatively (D3, IMP-025).
- **Behaviour-preservation gate** — touching shared machinery (`meta.rs`) keeps
  existing suites green unchanged (D2).

---

## 4. Architecture & module boundaries

Review mirrors **`slice`** (raw `entity::Kind`, bespoke commands, derived status,
`state_dir`), **not** `adr` (the `GovKind` status-based spine does not fit a
derived-status kind) — **D1**.

### New modules (2)

**`src/contentset.rs`** — pure leaf (ADR-001 leaf tier), the IMP-025 candidate
primitive. Consumer-agnostic, liftable — **D3**.

```rust
pub struct ContentSet(BTreeMap<String, String>);          // root-rel path → hash, ordered
pub struct SetDrift { pub changed: Vec<String>, pub added: Vec<String>, pub removed: Vec<String> }
impl ContentSet {
    pub fn diff(&self, current: &ContentSet) -> SetDrift;     // pure
    pub fn is_stale_against(&self, current: &ContentSet) -> bool;
}
pub fn compute(root: &Path, paths: &[String]) -> io::Result<ContentSet>; // impure shell
```

`compute` omits an absent path → `diff` reports it `removed` → stale (R1
absence⇒stale, §9). Uses `sha2` directly (a one-liner) rather than depending on
the impure `git.rs` seam — keeps the leaf liftable; trivial two-call-site
duplication accepted (D3).

**`src/review.rs`** — command tier, one-file pure-core + impure-shell demarcation
(the `worktree.rs` shape, not `adr.rs`'s thin forwarder):

- *pure core*: `Facet`/`FindingStatus`/`Severity`/`Role`/`ReviewStatus` enums
  (each `as_str` + const array + drift canary); `derived_status` (§8);
  `can(verb, from, role)` transition predicate (§5); render fns (`toml_string`
  escaped).
- *impure shell*: `REVIEW_KIND: entity::Kind` (`dir=".doctrine/review"`,
  `prefix="RV"`, scaffold); verb handlers; baton/lock/cache coordination
  (`with_turn`, §6); thin `run_*` forwarders. Split a `review_state.rs` later only
  if the shell grows unwieldy.

### Wiring (hardcoded match sites — clap dispatch is not data-driven)

| File | Change |
|---|---|
| `integrity::KINDS` | `KindRef { kind: &REVIEW_KIND, stem: "review", state_dir: Some(".doctrine/state/review") }` — 2nd kind with a `state_dir` |
| `src/main.rs` | `mod review; mod contentset;` · `Command::Review` · `ReviewCommand` enum (nested, memory-style) · arms in `conduct_on_command` (Write/Read) + `execute` |
| `install/manifest.toml` `[dirs].create` | `.doctrine/review` |
| `.gitignore` | `!.doctrine/review/` (`mem.pattern.install.authored-entity-wiring`) |
| `install/templates/review.{toml,md}` | embedded (`mem.pattern.build.rust-embed-no-rerun` — touch embedding crate to re-embed) |

### Coupling (ADR-001, no cycles)

`contentset` (leaf) ← `review`. `review` (command) → `entity`, `fsutil`,
`contentset`, `tomlfmt`, `listing`, `sha2`. Close-gate: `slice`-close shell →
`review::unresolved_blockers_for` (one-way; `review` never imports `slice`) — §7.
The spec `Registry` is **not** involved (it is spec-validation-specific) — §7.

---

## 5. Authored schema & finding lifecycle

### Authored toml

Top-level engine meta (`id`/`slug`/`title`) **but no `status`** — D-C8 derives it
and the storage rule forbids derived data in authored files. Nothing stored =
nothing to diverge (designs out the SL-009 problem).

```toml
id    = 7
slug  = "design-review-of-sl-024"
title = "design review of SL-024"
# (no status — derived, D-C8)

[review]
facet     = "design"     # closed enum, D-C11 (no `drift`)
raiser    = "reviewer"
responder = "author"

[target]                 # outbound edge (ADR-004): RV-007 ──reviews──▶ SL-024
ref   = "SL-024"
phase = "PHASE-03"       # optional; phase-scoped facets only

[[finding]]
id          = "F-1"      # raiser-owned, fixed, bare doc-local id, append-only
status      = "open"     # open|answered|contested|verified|withdrawn
severity    = "major"    # raiser-owned, fixed (blocker|major|minor|nit)
title       = "..."      # raiser-owned, fixed
detail      = "..."      # raiser-owned, fixed
disposition = "fixed"    # responder-owned, mutable
response    = "..."      # responder-owned, mutable
```

### `Meta.status` accommodation — **D2** (touches shared machinery)

`scan_kind` reads only `.id`, but strict `Meta` fails on a missing `status`.
**Decision: `#[serde(default)]` on `Meta.status`** — one-line, behaviour-preserving
(existing kinds always emit `status`; the default fires only for review's
intentional omission), + a canary that a status-less toml parses → `""`. Rejected
alternative: an id-only reader — branches `KINDS` or narrows the "malformed toml
is a hard error" guarantee for all kinds. *Flagged for inquisition: contract
widening vs purity.*

### Field ownership (D-C5) — keys disjoint

| Field | Owner | Mutability |
|---|---|---|
| `id` `title` `detail` `severity` | raiser | fixed at raise (append-only identity) |
| `disposition` `response` | responder | mutable |
| `status` | transition graph | single-owner edges only — never free-edited |

Append-only: findings never deleted/renumbered; `id = F-<n>`, next = `max+1`
(bare doc-local form).

### Transition graph — pure `can(verb, from, role)`

| verb | from | role | → |
|---|---|---|---|
| `raise` | (none) | raiser | open |
| `dispose` | open \| contested | responder | answered |
| `verify` | answered | raiser | **verified** (terminal) |
| `contest` | answered | raiser | contested |
| `withdraw` | open \| answered | raiser | **withdrawn** (terminal) |

This per-finding predicate **is** D-C4's "refuse out-of-turn write"; `--as`
asserts the role (cooperative, not security — ADR-007 Negative).

### Contest/verify rationale — **D10** (ADR schema gap)

The schema has no raiser-owned *mutable* field, so `contest`'s rationale has no
authored home (the raiser cannot edit `detail`/`response`). **Decision: `--note`
on contest/verify lands in the baton handoff log (ephemeral)** — ADR-007 Neutral
("handoff chatter → baton; durable → promoted to a finding/disposition"). Durable
rationale the raiser promotes to a new finding. *Flagged: alternative is a new
raiser-owned mutable field, a schema expansion beyond ADR-007 → would need
`/consult`.*

### md companion (D-C6) + edit mechanics

`review-NNN.md`: `## Brief` seeded at `new` (pre-reading, lines of attack) +
optional `## Synthesis`. Creation: `review new --facet --target [--phase]` writes
the authored ledger (empty findings) + seeds `## Brief`; `raise` appends. Empty
ledger is the real `active`/`await=raiser` state (D-C8). Appends + transitions use
`toml_edit` DocumentMut (edit-preserving, `governance.rs:290` pattern) extended to
finding-scoped edits; user free-text spliced via `toml_string`.

---

## 6. Runtime coordination (baton / lock / cache) — **D5, D6**

Locus = **parent tree** (R1/D4): `.doctrine/state/review/NNN/`.

**D-C1/D-C7 reconciliation.** D-C1 says the baton is *per-worktree*; D4 puts the
locus in the parent. No contradiction: under the dispatched-fork model (§4) the
review's single working tree **is** the parent (orchestrator-sole-writer, ADR-006)
— forks are *read* for hashing, never co-write the ledger. So there is exactly one
baton, per-(review-locus)-worktree, and D-C7 ("never a shared ledger across
worktrees") holds by construction.

```
baton.toml   await (cached role) · authored_hash (CAS key) · rounds · contest counts · handoff log
cache.toml   review context cache (§9)
lock         create_new mutex — pid+timestamp for diagnostics
```

Separate files (**D5**, OQ-1): the lock must be a distinct OS artefact; the cache
may be large; `baton` is small and recomputed from the authored ledger. D-C2
split: `await` is cached-derivable; `rounds`/`contests`/`handoff` are non-derivable
observability bookkeeping (lost on baton loss — acceptable).

### The turn protocol — one `with_turn` wrapper (**D6**)

All five write verbs ride one higher-order seam — the single home of
D-C3/D-C4/D-C4a:

```rust
fn with_turn<F>(root: &Path, id: u32, verb: Verb, role: Role, f: F) -> anyhow::Result<()>
where F: FnOnce(&mut DocumentMut, &[Finding]) -> anyhow::Result<()>
```

**Responsibility split.** The wrapper owns coordination + the *static* verb→role
check (knowable without a finding: raise/verify/contest/withdraw⇒raiser,
dispose⇒responder). The closure `f` owns the *per-finding* transition — only the
verb knows its target finding id, so it runs `can(verb, finding.status, role)`
(§5/§8) and applies the edit. The wrapper does not (cannot) gate per-finding.

```
1. acquire lock   create_new(lock) — AlreadyExists ⇒ bail "RV-NNN busy; re-run"
                  LockGuard::drop removes it (normal + panic; NOT hard-kill)
2. read authored  parse review-NNN.toml → findings
3. CAS guard      sha256(authored) ≠ baton.authored_hash ⇒ recompute await,
                    rewrite baton, bail "ledger changed underneath — re-run"
                    (missing baton ⇒ recompute, treat cold, proceed)
4. static role    asserted --as role == verb's required role; mismatch ⇒ bail (D-C4)
── mutate ──
5. AUTHORED FIRST f(doc, findings) runs per-finding can() then applies the edit;
                    write_atomic                                       (D-C3)
6. recompute      (_, new_await) = derived_status(new); new_hash = sha256(new authored)
7. BATON LAST     write_atomic baton{ await: new_await, authored_hash: new_hash, rounds+1, … }  (D-C3)
8. release        LockGuard drop
```

### Two guards, two jobs (the D-C4a interpretation, **D6**)

- **Lock = serialize concurrent invocations.** `create_new`; held *within one
  invocation only* (turns persist via the baton, not the lock) → ms lifetime.
- **CAS = detect *out-of-band* hand-edits, not concurrent ones.** The lock already
  prevents concurrent clobber, so "ledger changed since the read" (D-C4a) can only
  mean a human hand-edited the authored toml between invocations → the cached
  `await` is stale → recompute from authored truth (D-C2), abort, re-run (re-gates
  the verb if the edit changed whose turn it is). *Interpretation flagged — lock +
  CAS both present and named in D-C4a; this assigns each its job.*

### Crash & concurrency

| Event | Outcome |
|---|---|
| Crash between 5–7 | authored has edit, baton stale → next call CAS self-heals (D-C3 resumable) |
| Concurrent call | 2nd loses `create_new` race → clean abort, no clobber (D-C4a test) |
| Out-of-band edit | CAS catches → baton refresh + re-run |
| Hard kill (`-9`) | stale lock → `review unlock` escape hatch (pid+time aids diagnosis) |

---

## 7. `reviews` edge + reverse close-gate — **D8**

**Forward edge** (authored, ADR-004): `[target].ref` (+ optional `phase`) *is* the
edge — no separate file. `review show` renders `RV-007 ──reviews──▶ SL-024`.
Target validated at `review new` (reuse `integrity::parse_canonical_ref` /
`kind_by_prefix`) — dangling ref refused early; `[target].phase` existence-check
deferred (minor).

**Reverse close-gate** (D-C9b) — standalone scoped scan in `review.rs`, **not**
the spec `Registry` (wrong cohesion — that index is spec-validation-specific), and
no general reverse index (scope non-goal):

```rust
struct BlockerRef { rv: String, finding: String }   // "RV-007" / "F-2"
fn unresolved_blockers_for(root: &Path, subject_ref: &str) -> anyhow::Result<Vec<BlockerRef>>
//   unresolved = severity==Blocker && status ∉ {verified,withdrawn}
//   active     = derived_status == Active (D-C8)
```

Scans `.doctrine/review/*`, matches `[target].ref`, filters findings — pure check
+ thin scan shell (the `integrity::scan_kind` shape). Cost O(#RV)/close (**R2**:
fine at scale; note indexing later).

**Injection — the close command shell, not `set_slice_status`.** The gate fires
only on the closure-seam moves (`audit→reconcile`, `reconcile→done`; seam at
`slice.rs:471`): read `from`; if crossing the seam, call
`unresolved_blockers_for`; non-empty ⇒ bail listing `RV-NNN/F-n`. Keeps the FSM
writer focused and isolates the one-way `slice-shell → review-query` coupling in
the impure layer. Teeth live in the **binary** (the `slice status …` refusal), not
skill prose. Gate implemented for **slice close** only (pilot path); other-target
close-gates are future.

---

## 8. Derived status function (D-C8) — **D7**

```rust
enum Await { Raiser, Responder, None }
fn derived_status(findings: &[Finding]) -> (ReviewStatus, Await) {
    if findings.is_empty() { return (Active, Raiser); }            // raiser first — empty ≠ done
    if findings.iter().any(|f| matches!(f.status, Open | Contested)) { return (Active, Responder); }
    if findings.iter().any(|f| f.status == Answered) { return (Active, Raiser); }
    (Done, None)                                                   // all ∈ {Verified, Withdrawn}
}
```

Total over the status enum; empty → `Active` (the SL-009-divergence-proof case).
Never stored — computed at `show`/`list`/`status`/close-gate; the baton caches
only `await` (D-C2).

### `await` summarizes; `can()` gates — **D7** (refines D-C8 turn semantics)

A global "role == await" gate is **wrong**: after the raiser raises F-1 (open),
`derived` says `await=Responder`, which would forbid the raiser raising F-2 —
breaking normal batch-raising. Resolution: the turn gate is **per-finding
`can()`** (§5); `await` is a derived *convenience* (baton cache, `status` display,
handoff routing) that does **not** independently block a verb.

- `raise` valid whenever `role==Raiser`, *even while `await=Responder`*
  (append-only, raiser-owned).
- `dispose`/`verify`/`contest`/`withdraw` gated by `can()` — the single-owner edges.
- With mixed states (F-1 open, F-2 answered) both roles legitimately have work;
  `await` is a *priority summary* (open/contested ⇒ Responder wins display), never
  exclusive. `rounds` is a coarse counter (bump on summarized-role flip).

---

## 9. Review context cache + `prime` (D-C10) — **D9**

`cache.toml` (runtime, regenerable, never authored) — the reviewer's *learned*
model; **not** the LLM token cache (T-b: doctrine makes no attempt to observe
token-cache warmth).

```toml
[[area]]                          # domain_map: area → purpose → curated paths (T-a)
name = "turn protocol"; purpose = "baton/lock/CAS serialize turns"
paths = ["src/review.rs", "src/state.rs"]
[[invariant]] text = "await derived, never stored (D-C8)"
[[risk]]      text = "stale baton after out-of-band edit"
[hashes]                          # the ContentSet (§4) — the staleness key
"src/review.rs" = "<sha256>"
```

- **Curated load-bearing set** (T-a) — not a mechanical read-log. `[hashes]` =
  `ContentSet` over `⋃ area.paths`, the comparison baseline.
- **Staleness — `current` vs `stale`** (T-b naming):
  `stored.diff(compute(parent_root, ⋃ paths))`. Empty ⇒ `current`, else `stale`
  (lists drifted paths). An optimization signal, **not a gate**; surfaced by
  `review status`.
- **R1 absence⇒stale**, **single parent root**; a future `subject` root
  (pre-import fork review) is additive on the `(root, relpath, hash)` shape →
  **IMP-024**.
- **T-c**: file-level content-hash is the key (position-independent); region
  anchors advisory-only → **IDE-002**.
- **`prime` flow** (T-a): `review prime RV-NNN --seed` emits git-changed candidates
  (starting point, not authority); `review prime RV-NNN` (domain_map on
  stdin/`--from`) validates → `contentset::compute` → writes `cache.toml`. Read-class
  for *authored* conduct (no authored mutation), but **acquires the per-review lock**
  to serialize the `cache.toml` write against a concurrent `prime`/`status`.
  Read-hook attestation is a future seeding aid, not authority (T-a).

---

## 10. `/audit` pilot rewire — **D11**

`/audit`'s finding/disposition/closure loop *is* the RV ledger; the rewire closes
the scaffold gap structurally (D-C0). One skill only — `/inquisition`,
`/code-review`, reconciliation = **IMP-023** (non-goal).

| `/audit` today (`audit.md`) | RV |
|---|---|
| author `audit.md` | `review new --facet reconciliation --target SL-NNN` + `prime` |
| finding (expected/observed/evidence) | `raise` → severity/title/detail |
| disposition (aligned/fix-now/design-wrong/follow-up/tolerated) | `dispose` (free-text) + `verify` |
| finding raised in error | `withdraw` |
| "no undispositioned findings before close" (prose) | **D-C9a/b teeth** |
| `audit.md` prose | `## Synthesis` (D-C6) |

The disposition taxonomy becomes the recommended `disposition` vocabulary;
**`blocker`** severity is the only one that gates `/close` (D-C9b). A self-audit
drives both roles via `--as` (cooperative); lock + `can()` keep it correct for one
or two parties. `audit.md` retired for new audits (existing files remain — no
migration). Skill source under `plugins/doctrine/skills/audit/SKILL.md`; re-embed
via `doctrine skills install` + touch `src/skills.rs`.

---

## 11. Decisions

- **D1** — review mirrors `slice` (raw `entity::Kind`), not `adr` (`GovKind`):
  derived status, bespoke commands, `state_dir`.
- **D2** — `#[serde(default)]` on `Meta.status` (vs id-only reader). *Flagged.*
- **D3** — `contentset` standalone leaf, owns `sha2`; promotion deferred → IMP-025.
- **D4** — review locus = parent tree (ADR-006 dispatched-fork model); domain_map
  single parent root; subject-root additive → IMP-024 (resolves R1).
- **D5** — runtime layout: separate `baton.toml`/`cache.toml`/`lock` (OQ-1).
- **D6** — concurrency: `create_new` lock (RAII) + `sha256` CAS as the
  out-of-band-edit detector; `with_turn` wrapper, authored-first/baton-last
  (OQ-2; interprets D-C4a). *Flagged.*
- **D7** — `await` is a derived summary; the gate is per-finding `can()`; `raise`
  is not await-blocked (refines D-C8 turn semantics).
- **D8** — close-gate = standalone scoped scan in `review.rs`, injected in the
  close command shell (not the registry, not `set_slice_status`).
- **D9** — domain_map = curated set (T-a); file-level content-hash staleness,
  absence⇒stale; `current`/`stale` naming (T-b); region anchors advisory → IDE-002
  (T-c).
- **D10** — contest/verify rationale → ephemeral baton handoff note (vs schema
  field). *Flagged.*
- **D11** — `/audit` pilot; disposition taxonomy → RV disposition vocabulary; one
  skill only (IMP-023 defers the rest).

## 12. Open questions / flagged for inquisition

- **D2, D6, D10** carry locked leans but are explicitly flagged as the
  interpretation/expansion points an adversarial pass should probe. No question
  remains unresolved without a recommendation.
- All deferrals have backlog homes: IMP-022 (Drift Ledger), IMP-023 (skill
  rewiring), IMP-024 (large-review funnel / subject-root), IMP-025 (promote
  `contentset`), IDE-002 (durable region anchor).

## 13. Risks

- **R1 — warm-cache × worktree.** *Resolved* (D4): locus=parent collapses the
  multi-root concern; domain_map is single-rooted `(path, hash)`.
- **R2 — close-gate scan cost** O(#RV)/close. Acceptable now; note indexing later.
- **R-a — `Meta.status` widening** weakens the shared contract slightly (a
  status-less non-review toml parses `""`). Mitigated: integrity reads only `.id`;
  canary pins the behaviour (D2).
- **R-b — stale lock on hard-kill.** `review unlock` escape hatch; RAII covers
  panic/normal paths.
- **R-c — `with_turn`/CAS is greenfield concurrency** — the highest-risk code;
  the concurrent-invocation no-clobber test (VT) is the proof obligation.
- **R-d — single-agent dual-role audit ergonomics** — `--as` toggling is verbose;
  acceptable for the pilot (the structured record + teeth are the win).

## 14. Verification matrix

Mode: **VT** by test, **VA** by agent, **VH** by human. Phase assignment is
`/plan`'s job.

### Pure core (unit)
| Obligation | Ref | Mode |
|---|---|---|
| `derived_status` total over enum | D-C8 | VT |
| named status cases (empty→Active/Raiser; open→Responder; answered→Raiser; all-verified/all-withdrawn/mixed-terminal→Done/None; open+answered→Responder) | D-C8 | VT |
| `can()` single-owner edges — valid pass, wrong-role/state refuse | D-C5 | VT |
| facet enum closed, no `drift`; all enum↔array canaries | D-C11 | VT |
| `contentset::diff` + absence⇒stale | R1 | VT |

### Authored ledger
| Obligation | Ref | Mode |
|---|---|---|
| field ownership disjoint; raiser fields immutable | D-C5 | VT |
| append-only finding ids | D-C5 | VT |
| edit-preserving transitions (comments/unknown keys survive) | §4 | VT |
| render escaping (hostile title/detail round-trips) | render-splice | VT |
| `Meta.status` status-less toml parses → `""`; existing kinds unaffected | D2 | VT |
| dangling `[target].ref` refused at `new` | §7 | VT |

### Runtime coordination
| Obligation | Ref | Mode |
|---|---|---|
| **no-clobber under concurrency** (one wins, other aborts) | D-C4a | VT |
| CAS catches out-of-band edit → baton refresh + re-run | D-C4a/D-C2 | VT |
| authored-first/baton-last; crash-between resumable via CAS | D-C3 | VT |
| out-of-turn refuse; `raise` allowed while `await=Responder` | D-C4/§8 | VT |
| `status` rebuilds baton (cache == recompute) | §Verif | VT |
| baton in per-worktree (parent) gitignored state | D-C7 | VT/VH |

### Edge + close-gate
| Obligation | Ref | Mode |
|---|---|---|
| review-done = all findings terminal | D-C9a | VT |
| `/close` refuses on unresolved blocker; passes when none | D-C9b | VT |
| `unresolved_blockers_for` scan correctness | §7 | VT |
| gate fires only on closure seam | §7 | VT |

### Cache + integration
| Obligation | Ref | Mode |
|---|---|---|
| `prime` persists curated domain_map + `[hashes]`; `current` after | D-C10 | VT |
| staleness current/stale; drift lists path | D-C10/T-b | VT |
| install wiring: committable tree, manifest dir, KINDS row seen by `validate` | authored-entity-wiring | VT |
| **`/audit` produces an RV not `audit.md`** (e2e) | §10 | VA |
| `just check` green, clippy zero | conventions | VT/VH |
