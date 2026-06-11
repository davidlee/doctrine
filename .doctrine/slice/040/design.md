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

### `Meta.status` accommodation — **D2** (scan-path id-only reader)

`scan_kind` reads only `.id`, but strict `Meta` fails on review's intentionally
status-less toml. **Decision: a scan-path id-only reader** — `scan_kind`
deserialises a minimal `{ id }` (serde ignoring the rest) instead of the full
`Meta`. The shared `Meta` stays **strict**: a genuinely corrupt `adr`/`slice`/`spec`
toml with a missing `status` still hard-fails parse at every status-bearing reader
(`show`/`list`/render), preserving the "malformed metadata toml is a hard error"
contract. Review's own `show`/`list` live in `review.rs` and never ask the shared
reader for a status they know is derived (§8). Leniency is confined to the one path
that needs it. *Reverses the round-1 lean (`#[serde(default)]` on shared
`Meta.status`) per `inquisition.md` Charge III: widening the shared struct salted
the well for all kinds; the scoped reader does not.* Verify: a corrupt non-review
toml with missing `status` still hard-fails (the preserved invariant); review's
status-less toml scans for `.id` cleanly.

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

### Contest/verify handoff note — **D10** (ephemeral chatter, not rationale)

The schema has no raiser-owned *mutable* field. **Decision: `--note` on
contest/verify is explicit *handoff chatter* — it lands in the baton handoff log
(ephemeral, lost on baton loss, D-C2 bookkeeping tier)** — ADR-007 Neutral
("handoff chatter → baton; durable → promoted to a finding/disposition"). It is
**not** durable rationale and must not be named so (the misnomer invites users to
entrust durable justification to a discarded file — `inquisition.md` Charge V):
durable contest/verify justification the raiser **promotes to a new finding**, its
authored home. CLI help carries the same framing. *A raiser-owned mutable schema
field (durable rationale in the ledger) stays out of scope — a schema expansion
beyond ADR-007 → `/consult` if ever wanted.*

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

**D-C1/D-C7 reconciliation** (ADR-007 D-C1 clarification, SL-040). D-C1 says the
baton is *per-worktree*; D4 puts the locus in the parent. For the **pilot** these
coincide: review verbs operate on the **main/parent tree**, where parent ≡ worktree,
so D-C7 ("never a shared ledger across worktrees") holds directly — forks are *read*
for hashing, never co-write the ledger. This is the pilot invariant, **enforced**
(not asserted): a review verb whose resolved root is a fork **bails** —
fork-invoked review (the locus would resolve to a fork that `WITHHELD` keeps from
seeing the parent baton, `worktree.rs:71`) is **IMP-024**, not yet supported. The
guard lives in the impure shell (root resolution); reconciling "per-worktree" with
a parent-locus addressing rule is deferred to IMP-024.

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
3. ENTRY CAS      sha256(authored) ≠ baton.authored_hash ⇒ recompute await,
                    rewrite baton, bail "ledger changed underneath — re-run"
                    (missing baton ⇒ recompute, treat cold, proceed)
                    [catches edits landing BEFORE this invocation]
4. static role    asserted --as role == verb's required role; mismatch ⇒ bail (D-C4)
── mutate ──
5. AUTHORED FIRST f(doc, findings) runs per-finding can() then applies the edit.
                    PRE-WRITE CAS: re-read authored bytes; sha256 ≠ the step-2
                    snapshot ⇒ bail "ledger changed underneath — re-run" (do NOT
                    write). [catches a hand-edit landing DURING this invocation —
                    the lock excludes other invocations, not a human editor].
                    Else write_atomic                                  (D-C3)
6. recompute      (_, new_await) = derived_status(new); new_hash = sha256(new authored)
7. BATON LAST     write_atomic baton{ await: new_await, authored_hash: new_hash, rounds+1, … }  (D-C3)
8. release        LockGuard drop
```

### Two guards, two jobs (the D-C4a interpretation, **D6**)

- **Lock = serialize concurrent invocations.** `create_new`; held *within one
  invocation only* (turns persist via the baton, not the lock) → ms lifetime.
- **CAS = detect *out-of-band* hand-edits, not concurrent ones.** The lock prevents
  concurrent *invocation* clobber; it does **not** stop a human with a text editor
  (no invocation, no lock). So "ledger changed since the read" (D-C4a) means a
  hand-edit, and it must be caught in **both** windows: an edit *before* this
  invocation (entry CAS, step 3 — stale cached `await`, recompute from authored
  truth per D-C2, abort, re-run, re-gating the verb if the turn changed) **and** an
  edit *during* this invocation, between the step-2 read and the step-5 write
  (pre-write CAS, step 5 — abort before writing so the stale in-memory `DocumentMut`
  cannot overwrite newer authored truth). A single entry-only check would let the
  mid-turn edit through, and the next invocation's CAS would *not* catch it (baton
  hash == our clobber). Residual: a hash→`write_atomic` rename-instant TOCTOU
  survives, shrunk to the rename instant — **tolerated** (SL-040 ruling). *D-C4a
  names lock + CAS; this assigns each its job — the inquisition round-1 remedy
  (`inquisition.md` Charge I).*

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

*Shell-injection bypass risk* (`inquisition.md` Charge VIII): placing the gate in
the close shell, not `set_slice_status`, means any *future* caller crossing the
closure seam without the shell would evade it. Mitigation: a VT asserts the close
shell is the **sole** seam-crossing caller today (`set_slice_status` is invoked
across the `audit→reconcile`/`reconcile→done` seam from nowhere else); any new
seam-crossing caller MUST re-invoke `unresolved_blockers_for`. If a second caller
ever exists, reconsider moving the gate to the FSM.

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
- **D2** — scan-path id-only reader (`scan_kind` reads `{ id }`); shared `Meta`
  stays strict. *Round-1 inquisition reversed the prior `#[serde(default)]` lean
  (Charge III).*
- **D3** — `contentset` standalone leaf, owns `sha2`; promotion deferred → IMP-025.
- **D4** — review locus = parent tree (ADR-006 dispatched-fork model); domain_map
  single parent root; subject-root additive → IMP-024 (resolves R1). Pilot invariant
  **enforced**: review verbs refuse a fork-resolved root (ADR-007 D-C1 clarified —
  parent-locus pilot rule; per-worktree reconciliation deferred to IMP-024).
- **D5** — runtime layout: separate `baton.toml`/`cache.toml`/`lock` (OQ-1).
- **D6** — concurrency: `create_new` lock (RAII) + `sha256` CAS, fired **twice**
  (entry: pre-invocation edit; pre-write: mid-invocation edit) as the
  out-of-band-edit detector; `with_turn` wrapper, authored-first/baton-last
  (OQ-2; interprets D-C4a). *Round-1 inquisition added the pre-write CAS (Charge I).*
- **D7** — `await` is a derived summary; the gate is per-finding `can()`; `raise`
  is not await-blocked. *Ratified at the seal: ADR-007 D-C4 clarified (SL-040) —
  out-of-turn is enforced per-finding, `await` is not an independent gate.*
- **D8** — close-gate = standalone scoped scan in `review.rs`, injected in the
  close command shell (not the registry, not `set_slice_status`).
- **D9** — domain_map = curated set (T-a); file-level content-hash staleness,
  absence⇒stale; `current`/`stale` naming (T-b); region anchors advisory → IDE-002
  (T-c).
- **D10** — contest/verify `--note` is ephemeral *handoff chatter* (not "rationale",
  Charge V) → baton handoff log; durable justification promotes to a finding. A
  raiser-owned mutable schema field stays out of scope (→ `/consult` if ever wanted).
- **D11** — `/audit` pilot; disposition taxonomy → RV disposition vocabulary; one
  skill only (IMP-023 defers the rest).

## 12. Open questions / flagged for inquisition — **resolved round 1**

- Round-1 external hostile pass (codex / GPT-5.5) recorded in `inquisition.md`.
  Disposition: **D2 reversed** (Charge III — scan-path id-only reader, not
  `serde(default)`); **D6 hardened** (Charge I — pre-write CAS added); **D10
  reframed** (Charge V — handoff chatter, not "rationale"); §14 no-clobber VT
  strengthened to scripted interleavings (Charge VI); close-gate sole-caller VT
  added (Charge VIII). Charge VII (phantom §Verification ref) **dismissed** — the
  section exists (ADR-007:232).
- **Two ADR-007 tensions ratified at the seal** (not re-decided locally): D-C4
  clarified — out-of-turn enforced per-finding, `await` is a summary (Charge II);
  D-C1 clarified — parent-locus pilot rule + fork-root guard, per-worktree
  reconciliation deferred to IMP-024 (Charge IV).
- All deferrals have backlog homes: IMP-022 (Drift Ledger), IMP-023 (skill
  rewiring), IMP-024 (large-review funnel / subject-root), IMP-025 (promote
  `contentset`), IDE-002 (durable region anchor).

## 13. Risks

- **R1 — warm-cache × worktree.** *Resolved* (D4): locus=parent collapses the
  multi-root concern; domain_map is single-rooted `(path, hash)`.
- **R2 — close-gate scan cost** O(#RV)/close. Acceptable now; note indexing later.
- **R-a — shared `Meta` contract** kept strict (D2 reversed to a scan-path id-only
  reader). A corrupt non-review toml missing `status` still hard-fails; leniency is
  confined to `scan_kind`'s `{ id }` deserialise. VT pins both halves.
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
| scan-path id-only reader scans review's status-less toml for `.id`; shared `Meta` stays strict — corrupt non-review toml missing `status` still HARD-FAILS | D2 | VT |
| dangling `[target].ref` refused at `new` | §7 | VT |

### Runtime coordination
| Obligation | Ref | Mode |
|---|---|---|
| **no-clobber under concurrency** — scripted interleavings, assert FINAL LEDGER state (not just exit code): (a) two invocations, one wins / loser aborts then re-runs from refreshed baton and lands a correct turn; (b) crash between steps 5–7 → next call self-heals via entry CAS; (c) hand-edit landing AFTER the step-2 read → pre-write CAS aborts, no clobber (Charge I); (d) same-finding `contest` racing `verify` | D-C4a | VT |
| entry CAS catches pre-invocation edit → baton refresh + re-run; pre-write CAS catches mid-invocation edit → abort before write | D-C4a/D-C2 | VT |
| authored-first/baton-last; crash-between resumable via CAS | D-C3 | VT |
| out-of-turn refuse; `raise` allowed while `await=Responder` | D-C4/§8 | VT |
| `status` rebuilds baton (cache == recompute) | §Verif | VT |
| baton in per-worktree (parent) gitignored state | D-C7 | VT/VH |
| review verb refuses a fork-resolved root (IMP-024 not-yet-supported guard) | D4/D-C1 | VT |

### Edge + close-gate
| Obligation | Ref | Mode |
|---|---|---|
| review-done = all findings terminal | D-C9a | VT |
| `/close` refuses on unresolved blocker; passes when none | D-C9b | VT |
| `unresolved_blockers_for` scan correctness | §7 | VT |
| gate fires only on closure seam | §7 | VT |
| close shell is the SOLE seam-crossing caller of `set_slice_status` (no bypass) | §7 | VT |

### Cache + integration
| Obligation | Ref | Mode |
|---|---|---|
| `prime` persists curated domain_map + `[hashes]`; `current` after | D-C10 | VT |
| staleness current/stale; drift lists path | D-C10/T-b | VT |
| install wiring: committable tree, manifest dir, KINDS row seen by `validate` | authored-entity-wiring | VT |
| **`/audit` produces an RV not `audit.md`** (e2e) | §10 | VA |
| `just check` green, clippy zero | conventions | VT/VH |
