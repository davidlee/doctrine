# Design SL-178: Close drift-discharge legibility: richer error + skill recipe + shipped memory

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`doctrine slice status <id> done` refuses an undischarged-drift close with a
one-line error that names only the requirement ids. The fix — an `accept` REC
satisfying a 3-clause predicate — must be reverse-engineered from `src/slice.rs`
(`rec_discharges`). The recipe *is* captured, but only as a project-local memory
(`.doctrine/memory/items/mem_019f075f…`), unshipped: a CLI error pointing there
would violate POL-002 (platform independence from host-project state). Net cost
per governed close: ~4 round-trips (IMP-202, from SL-165 PIR S1/S2/S6).

Three independently-shippable legibility fixes collapse that cost:
1. enrich the close-gate error,
2. promote the recipe to a shipped master memory,
3. document the recipe (as a pointer) in the `/close` skill.

## 2. Current State

- **Error.** `src/slice.rs` `run_status` (`:823-839`): on the `reconcile → done`
  seam, `undischarged_drift` (`:1275`) returns `Vec<String>` (req ids only); the
  bail (`:831`) prints the id list plus a generic "reconcile each via an accept
  REC" tail. The authored status of each flagged REQ is loaded internally
  (`:1282`) but discarded.
- **Predicate.** `rec_discharges` (`:1331`) — correct, three clauses (a: move
  accept; b: a `status_delta` naming R with `to == authored`; c: `evidence_ref`
  ⊇ the residual coverage keys). Illegible from the error alone.
- **Recipe memory.** `mem.pattern.doctrine.close-drift-discharge-rec`
  (uid `mem_019f075f27d473718b0226bc2cb77dac`) lives in `.doctrine/memory/items/`
  (git-tracked local capture), carrying the wrong *class* signature for shipping:
  `scope.repo` set, `anchor.kind = checkout_state`, scope **tag-only**.
- **Shipped corpus.** Repo-root `memory/` is the RustEmbed source
  (`corpus.rs` `#[folder = "memory/"]`): real uid-named dirs +
  key-named symlinks aliasing them (e.g. `mem.pattern.doctrine.core-loop`
  → `mem_019e9a12…`). `memory sync` diffs the embed against the gitignored
  consumer cache `.doctrine/memory/shipped/`. `corpus.rs` `lint_master`/`is_inv`
  enforce the global-orientation INV signature + scope floor.
- **Skill.** `.agents/skills/close/SKILL.md` is the authored source
  (`.pi/`/`.claude/skills/close` symlink in; `.doctrine/skills/close` is the
  gitignored install copy). It mentions drift once (`:123`) but carries no
  discharge recipe.

## 3. Forces & Constraints

- **POL-002** — a shipped artefact (the CLI binary, a shipped skill/memory) must
  not reference host-project-local state. This is the whole reason Fix 3 precedes
  any error/skill pointer.
- **ADR-002** — a shipped memory must be the *global-orientation class*:
  `repo = ""` (admitted in every retrieval partition), `anchor_kind = none`
  (asserts nothing about a client's git), path-scoped (≥1 of
  paths/globs/commands — never tag-only), evergreen (no `reviewed`-driven
  staleness). The captured item violates all three signature fields.
- **ADR-005** — shipped knowledge is tiered: skills route, durable knowledge
  explains. One canonical source; other surfaces point.
- **STD-001** — no magic strings; the memory key referenced from `slice.rs` is a
  single named const.
- **ADR-001** — module layering; the error copy stays in the `slice` command
  shell (it already does), no new cross-module coupling.
- **Behaviour preservation** — the close-gate refuse/pass behaviour is the proof;
  it must stay green unchanged. Only the error *payload* and the
  `undischarged_drift` *return type* change.

## 4. Guiding Principles

- Legibility at the point of need, both reactive (the error, seen at refusal) and
  proactive (the skill/memory, read during `/reconcile`).
- Single maintained source of depth (the memory), cheap to update without a
  recompile; the other two surfaces point at it.
- Smallest change that satisfies the constraint; ride existing seams.

## 5. Proposed Design

### 5.1 System Model

Three independent surfaces, one shared identifier (the memory key):

```
slice status done  ──refuses──▶  error  ──points to──▶  ┌────────────────────┐
                                                        │ shipped master     │
/reconcile, /close skill  ──points to──▶ ──────────────▶│ memory (canonical, │
                                                        │ full recipe +      │
                                                        │ worked example)    │
                                                        └────────────────────┘
```

Canonical depth lives in the memory (D2). The error is self-sufficient for the
common case (names each REQ + status + the condensed 3-clause + the pointer);
the skill is a pointer-tier subsection.

### 5.2 Interfaces & Contracts

**Fix 1 — data shape (D1).** A module-local struct replaces the bare id vec:

```rust
struct UndischargedReq {
    req: String,
    authored: crate::requirement::ReqStatus,
}

fn undischarged_drift(root: &Path, id: u32) -> anyhow::Result<Vec<UndischargedReq>>
```

`authored` is already loaded at `:1282`; the loop pushes
`UndischargedReq { req, authored }` instead of `req`. Gate predicate stays
`!undischarged.is_empty()`.

**Fix 1 — const (STD-001).** Module-top, beside `SLICE_DIR`:

```rust
const CLOSE_DRIFT_RECIPE_MEMORY: &str = "mem.pattern.doctrine.close-drift-discharge-rec";
```

**Fix 1 — error copy** (bail at `:831`):

```
slice SL-178 → done: refused — undischarged residual drift:
  REQ-316 (authored: active)
  REQ-317 (authored: active)
discharge each with an `accept` REC owned by this slice, all three:
  (a) move = accept
  (b) a [[status_delta]] naming the REQ with to == its authored status above
  (c) [[evidence_ref]] ⊇ every coverage key feeding that REQ's composite
recipe + worked example: doctrine memory show mem.pattern.doctrine.close-drift-discharge-rec
```

(The key in the last line is the const, not a literal.)

**Fix 3 — re-class (ADR-002).** Rewrite `memory.toml` for the promoted master:

| field | from | to |
|---|---|---|
| `scope.repo` | `github.com/davidlee/doctrine` | `""` |
| `anchor.kind` | `checkout_state` | `none` |
| `git.commit/tree/checkout_state_id/base_commit/normalizer` | set | `""` |
| `repo_id_kind` | `remote` | `local_root` |
| `repo_id_confidence` | `high` | `low` |
| scope (paths/globs/commands) | tag-only | floor below |

```toml
[scope]
paths = [".doctrine/slice/", ".doctrine/rec/"]
globs = []
commands = ["doctrine slice", "doctrine rec"]
tags = ["area:close", "area:reconciliation"]
```

`memory_type = "pattern"` and the body (`memory.md`, the canonical recipe +
SL-165 → REC-093/094 worked example) ship unchanged.

**Fix 2 — skill subsection.** `.agents/skills/close/SKILL.md`, near `:123`: when
the gate fires, the condensed a/b/c, the `rec new --move accept …` line, and a
pointer `doctrine memory show mem.pattern.doctrine.close-drift-discharge-rec`. No
worked example duplicated (it lives in the memory, single source).

### 5.3 Data, State & Ownership

- The 3-clause recipe has one owner: the shipped master memory. Error and skill
  are derived pointers (condensed restatement is tolerated for the error's
  self-sufficiency; the worked example is never duplicated).
- The memory key is owned by one const in `slice.rs`; the skill and the master
  must spell the same key (verified by VT-2 + VA-1).
- The promoted memory has a single physical home after Fix 3:
  `memory/mem_019f075f…` (+ key symlink). The local
  `.doctrine/memory/items/mem_019f075f…` is removed — no double-load.

### 5.4 Lifecycle, Operations & Dynamics

Fix 3 mechanic (re-home + re-class, uid reused):
1. move content `.doctrine/memory/items/mem_019f075f…/` →
   `memory/mem_019f075f…/`;
2. add key→uid symlink
   `memory/mem.pattern.doctrine.close-drift-discharge-rec → mem_019f075f…`;
3. remove the local item (+ any slug alias);
4. re-class `memory.toml` per §5.2;
5. `memory sync` (dev binary) regenerates `.doctrine/memory/shipped/`;
   `memory find` then discovers it.

Uid reuse keeps the prose references in SL-165 PIR, IMP-202, RFC-011, and this
slice valid; no supersede chain.

### 5.5 Invariants, Assumptions & Edge Cases

- INV: the close-gate refuse/pass behaviour is unchanged (behaviour-preservation
  gate); the existing `vt1`/`vt2_*` cases prove it, updated only for the new
  return type + copy.
- INV: no shipped artefact references host-project-local state (POL-002) — the
  error/skill pointers resolve only after Fix 3 ships the master.
- Assumption: masters are hand-authored under `memory/` — there is no
  promote/ship verb (confirmed against the `memory` CLI surface).
- Edge: the const in Fix 1 and the shipped key in Fix 3 must match exactly; a
  mismatch makes the error's pointer dangle. Single const + VT-2 guard it.

## 6. Open Questions & Unknowns

(none open — D1/D2/D3 resolved in §7.)

## 7. Decisions, Rationale & Alternatives

- **D1 — error data shape: a named struct `UndischargedReq { req, authored }`.**
  `undischarged_drift` already loads the authored status; enriching the return is
  near-free and lets the error name status per REQ. Alternatives: a tuple
  (leaner, more opaque at the 3 call/test sites); re-read at the bail site
  (double-read, splits logic — rejected). Struct chosen for legibility + future
  extension.
- **D2 — tiering: memory canonical; error self-sufficient; skill points.** The
  recipe is needed both reactively (the error, at refusal) and proactively (the
  skill/memory, during reconcile to author RECs right the first time), so the
  error is not the sole point of need → memory is primary (durable, searchable,
  editable without a release, holds the worked example). But the error is
  reliably seen at the refusal moment and the slice's purpose is killing
  round-trips, so it carries the condensed full 3-clause inline + pointer.
  Alternative: pure-pointer error (one extra lookup, rejected); skill-canonical
  (inverts ADR-005, rejected).
- **D3 — promote by re-home + re-class, uid reused.** The captured item is the
  wrong *class* (ADR-002), not wrong content; re-class its signature and move it
  into the embed. Reuse the uid to keep existing prose references valid.
  Alternative: mint a new uid + supersede the old (audit trail, but breaks
  references and adds churn — rejected, no memory-graph backlinks exist).

Relation: SL-178 `related` IMP-216 — Fix 3 is the concrete first instance of
IMP-216's broad migration of ~46 project-local operational memories to shipped
reference knowledge.

## 8. Risks & Mitigations

- **R1 — three-way recipe drift** (error / skill / memory restate the 3-clause).
  Mitigation: D2 designates the memory canonical and forbids duplicating the
  worked example; error/skill carry only the condensed clauses + the shared key.
- **R2 — return-type ripple** to `vt1`/`vt2_*`. Mitigation: behaviour-preservation
  gate — the cases keep asserting refuse/pass; only the payload assertions change.
- **R3 — re-class misses an INV field** → `lint_master`/`is_inv` reject the
  master and `memory find` can't see it. Mitigation: VT-2 exercises the full
  sync → find path; the §5.2 table enumerates every field.
- **R4 — dangling pointer** if the const and the shipped key diverge. Mitigation:
  single const; VT-2 asserts the key resolves.
- **R5 — P1 releases ahead of P2** → the binary error points at an unshipped key
  (POL-002 violation in the interim). Mitigation: landing order P2 → P1 (§9).

## 9. Quality Engineering & Validation

| id | mode | criterion |
|---|---|---|
| VT-1 | test | `slice status <id> done` on a drifted slice errors naming each undischarged REQ with its authored status, the 3-clause, and the memory-key pointer (extends `vt1`/`vt2_*`) |
| VT-2 | test | shipped master `mem.pattern.doctrine.close-drift-discharge-rec` exists and is discoverable via `memory find` after `memory sync` (requires INV + scope-floor pass) |
| VA-1 | agent | `/close` skill carries the drift-discharge subsection pointing at the memory |
| VA-2 | agent | no shipped artefact references a local project memory (POL-002) |

Behaviour-preservation anchor: gate refuse/pass behaviour unchanged.

### Phasing

- **P2** — promote/re-class memory (`memory/`, remove local item). Independent;
  lands first.
- **P1** — error + data shape (`slice.rs`, tests). Codeable independently, but its
  binary error references the shipped key, so it must **not release ahead of P2**
  (POL-002 — see R5).
- **P3** — skill subsection. Points at the shipped key; lands after P2.

Landing order: **P2 → {P1, P3}**. The Fix 1 const and the Fix 3 key must match.

## 10. Review Notes

Internal adversarial pass (author):

- **F1 (fixed) — release ordering.** P1's binary error names the shipped key;
  honest only once P2 ships. Phasing tightened to P2 → {P1, P3} (§9); R5 added.
- **F2 (no change) — embed visibility.** `rust-embed` carries `debug-embed`, so
  debug/test builds read `memory/` from disk at runtime — VT-2 is feasible and a
  hand-added master is live via `./target/debug/doctrine`. The release PATH
  binary (`~/.cargo/bin/doctrine`) only sees a new master after reinstall; use
  the debug binary during P2.
- **F3 (test guard) — VT-1 brittleness.** Assert on substrings (each req id,
  `authored:`, the status token, `accept`, the memory key), not the exact
  multi-line copy, so the case survives copy edits.
- **F4 (VA method) — VA-2.** Verify by grepping the shipped surfaces — error
  literals in `src/`, shipped skills, shipped masters under `memory/` — for
  `.doctrine/memory/items/` and local-only uids; expect none.
- **F5 (author's call) — Unicode in copy.** The error uses `⊇` (and the existing
  `→`). Acceptable; an ASCII `superset of` would be friendlier to grep/test
  matching. Left to implementation taste.
- **F6 (confirmed) — mechanic.** `embedded_assets` (`corpus.rs:288,310`) admits
  only uid dirs and skips `mem.<key>` alias symlinks; key lookup resolves via the
  `memory_key` field. The §5.4 mechanic (real content in the uid dir + key
  symlink alias) is correct.

Doctrinal pass: POL-002 (R5/§9), ADR-002 (§5.2 re-class table), ADR-005 (D2
tiering), STD-001 (single const), ADR-001 (error stays in the slice shell) — all
satisfied. No governance conflict found; no `/consult` trigger.
