# Design SL-128: deliver_to config as single trunk-ref source

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The trunk *delivery ref* — the branch dispatch advances its audited code onto, and
the branch the SL-126 close-integration gate checks against — is hardcoded as
`refs/heads/main` in two places that must agree:

- the gate const `src/slice.rs:447 TRUNK_REF`, and
- close-skill prose (`plugins/doctrine/skills/close/SKILL.md`, the `--trunk
  refs/heads/main` literals at lines 74, 94–95).

IMP-124: introduce `[dispatch] deliver_to` in `doctrine.toml` as the single source
of truth for that ref, default `refs/heads/main`, consumed by both seams. The
change must be **behaviour-preserving** (default unchanged) and must not foreclose
later **PR-based delivery** (push a candidate as a PR rather than merging locally).

## 2. Current State

**Two distinct trunk concepts already live in the code — keep them apart:**

1. **Trunk *base* (fork-point) resolution** — `src/git.rs::trunk_tree_ish` /
   `trunk_ladder`. Resolves the *commit-ish* dispatch forks **from**. Precedence:
   `DOCTRINE_TRUNK_REF` env (explicit, hard-errors if unresolvable) → ladder
   `origin/HEAD → main → master` folded through `freshest_descendant`. Governed by
   ADR-006 D3 + SPEC-022. Returns a *commit sha*.

2. **Trunk *delivery target*** — the ref dispatch advances **to** / the gate checks.
   Surfaces: `--trunk: Option<String>` on `dispatch sync` (`src/main.rs:1048`); the
   SL-126 gate const `TRUNK_REF`. Currently `refs/heads/main`, by literal.

`deliver_to` is concept **#2 only**. #1 stays sealed (D1).

**Consumer mechanics:**

- **SL-126 gate** (`run_status`, `slice.rs:414`): on `reconcile → done`, calls
  `ledger::trunk_integration(&root, id, TRUNK_REF)`. `ledger` is ref-agnostic — the
  ref is supplied by the shell (`ledger.rs:444,460`). `run_status` already reads
  `doctrine.toml` (`load_conduct`, `slice.rs:453`).
- **`dispatch sync --integrate`** (`dispatch.rs::integrate`): plans a trunk row
  **only when `--trunk` is `Some`** (`dispatch.rs:1225`, `trunk.filter(...)`).
  Absent `--trunk` ⇒ **edge-only projection, trunk untouched** — a *live, tested*
  path (`tests/e2e_dispatch_sync.rs:688`, `e2e_dispatch_candidate.rs:1749` run
  `--integrate --edge <ref>` with no `--trunk`).
- **`dispatch sync --show-journal-trunk-oid`**: pure read of the journal row whose
  `target_ref == trunk`. Clap currently `requires = "trunk"` (`main.rs:1040`); the
  handler errors if absent (`main.rs:4314`).

**Config substrate:** `DispatchConfig` (`src/dispatch_config.rs`), `#[derive(Default)]`,
wired through `dtoml.rs` as `doc.dispatch`. Existing keys
`preferred-subprocess-harness`, `claude-force-subprocess-dispatch`. The
`DoctrineToml.dispatch` field carries `#[cfg_attr(not(test), expect(dead_code,
reason = "consumed by a future dispatch-config display slice"))]` (`dtoml.rs:38`)
— **this slice is that future consumer** (R5).

**Prose literal inventory** (`close/SKILL.md`): `refs/heads/main` at lines 74, 95,
96 (delivery — in scope), 102/107 (explanatory + TODO — in scope), and **line 68
`--base refs/heads/main`** — the *fork base* (concept #1, ADR-006 D3),
**deliberately out of scope** (D1 boundary).

## 3. Forces & Constraints

- **ADR-001 layering** — read config in the impure shell, pass the ref down; the
  `ledger` leaf stays ref-agnostic (no new coupling).
- **ADR-006 D3** — the base resolver (`trunk_tree_ish`, env + ladder) is canon;
  this slice does **not** touch it.
- **ADR-012** — dispatch integration topology; delivery is the trunk-write axis.
- **Behaviour-preservation gate** — the edge-only `--integrate` path and the
  existing `trunk_integration` suites must stay green **unchanged** (the proof).
- **pure/imperative split** — env/disk reads in the shell only.
- **No parallel implementation / DRY** — one config-read seam; don't fork a second
  `load_*` near-duplicate of `load_conduct`.

## 4. Guiding Principles

- **One source for the delivery ref.** `deliver_to` is it; everything else is a
  default or an explicit per-invocation override.
- **Delivery is its own axis, decoupled from base resolution.** That decoupling is
  exactly what later enables PR delivery (`deliver_to` becomes the PR *base*).
- **Config defaults the READS, never the WRITE opt-in.** The `--integrate`
  `--trunk`/`--edge` Options are load-bearing; leave them verbatim.

## 5. Proposed Design

### 5.1 System Model

```
doctrine.toml [dispatch] deliver_to = "refs/heads/main"   (NEW; default-valued)
        │
        ├─► SL-126 gate (slice.rs)            read  → trunk_integration(ref)
        ├─► dispatch sync --show-journal-..   read  → default for absent --trunk
        └─► dispatch deliver-to (NEW verb)    read  → prints resolved ref (option b)

dispatch sync --integrate  → --trunk/--edge Option semantics UNCHANGED (write opt-in)
close prose write line      → --trunk "$(doctrine dispatch deliver-to)"   (α hybrid)
```

### 5.2 Interfaces & Contracts

**Config field** (`dispatch_config.rs`):

```rust
const DEFAULT_DELIVER_TO: &str = "refs/heads/main";
fn default_deliver_to() -> String { DEFAULT_DELIVER_TO.to_string() }

pub(crate) struct DispatchConfig {
    // …existing fields…
    /// The trunk delivery ref dispatch advances to / the close-integration gate
    /// checks against (IMP-124). The PR *base* under a future delivery-mode key.
    /// NOT the fork-base resolver (ADR-006 D3 `DOCTRINE_TRUNK_REF`/ladder).
    #[serde(default = "default_deliver_to")]
    pub(crate) deliver_to: String,
}
```

`#[derive(Default)]` is **dropped** in favour of a hand-written `impl Default`
(non-empty default; the derive would yield `""`). Both the Rust `Default` path
(dtoml file-absent fallback) and serde absent-key path must yield
`refs/heads/main` — a parity invariant (I1).

**CLI** (`main.rs` / `dispatch.rs`):

- `dispatch deliver-to` — NEW thin read verb; prints the resolved `deliver_to` to
  stdout, newline-terminated. No flags beyond `--path`. (OQ-1 resolved: a
  `dispatch` subcommand, not a generic `config get`.)
- `dispatch sync --show-journal-trunk-oid` — `requires = "trunk"` **removed**;
  absent `--trunk` resolves from `deliver_to`.
- `dispatch sync --integrate` — **unchanged.** `--trunk`/`--edge` stay opt-in;
  absent trunk = edge-only.

**Precedence (delivery target):**

| Consumer | Precedence |
|---|---|
| `dispatch sync --show-journal-trunk-oid` | explicit `--trunk` › `deliver_to` › default |
| SL-126 gate | `deliver_to` › default |
| `dispatch sync --integrate` (write) | explicit `--trunk` only (no config default — preserves edge-only) |

`DOCTRINE_TRUNK_REF` env: **base resolution only, unchanged.** Not consulted for
delivery (it resolves to a commit-ish; delivery needs a writable ref — D3).

### 5.3 Data, State & Ownership

- `deliver_to` lives in `doctrine.toml [dispatch]` — authored project config, not a
  `.doctrine/` entity. Owned by the operator.
- **Single read seam (DRY).** Extract `load_doctrine_toml(root) -> DoctrineToml`
  in `slice.rs` (absent file → `DoctrineToml::default()`; `DoctrineToml: Default`
  confirmed, `dtoml.rs:18`). `load_conduct` stays as a **thin delegating wrapper**
  (`load_doctrine_toml(root)?.conduct`) so its other caller (`slice.rs:1083`) and
  tests (`2419–2450`) stay green. The gate reads `load_doctrine_toml(&root)?.
  dispatch.deliver_to` (one parse in `run_status`, reused for `.conduct` at
  `slice.rs:428`).
- **R5 — drop the dead-code expectation.** Reading `doc.dispatch` in non-test code
  makes the `#[cfg_attr(not(test), expect(dead_code, …))]` on `DoctrineToml.dispatch`
  unfulfilled → a compile error. Remove that attribute as part of wiring the gate.
- The `dispatch sync` handler (`main.rs`) resolves `--trunk` for the read stages
  from the same `DispatchConfig` (already reachable via dtoml parse).
- `ledger` ownership unchanged: ref-agnostic, ref passed in.

### 5.4 Lifecycle, Operations & Dynamics

1. Operator sets (or omits) `[dispatch] deliver_to`. Absent ⇒ `refs/heads/main`.
2. `/close` step-3a: write line `dispatch sync --integrate --trunk "$(doctrine
   dispatch deliver-to)"`; verify line `--show-journal-trunk-oid` may omit
   `--trunk` (defaults from config) or keep the verb for symmetry.
3. `reconcile → done`: gate reads `deliver_to`, checks integration, refuses on
   unintegrated dispatched code (semantics unchanged).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 — default agreement.** `DispatchConfig::default().deliver_to ==`
  serde-absent `==` `refs/heads/main`.
- **I2 — edge-only preserved.** `--integrate` with no `--trunk` plans no trunk row.
- **I3 — explicit override.** A passed `--trunk` always wins over config.
- **Bad ref:** `deliver_to` naming a nonexistent ref surfaces at git use-time
  (gate `resolve_ref` → "trunk ref … unresolved"; integrate FF-CAS). No new
  validation surface (matches A1 / out-of-scope).
- **Empty string:** `deliver_to = ""` is operator error; git surfaces it. Not
  specially handled.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved, D5)** — verb shape: `doctrine dispatch deliver-to`.
- **OQ-2 (resolved, D3)** — precedence: explicit flag › config › default; env
  base-only.
- **Residual (out of scope, follow-up)** — PR/remote *delivery mode* (`[dispatch]
  delivery-mode = "merge" | "pull-request"` + remote/refspec). `deliver_to`
  becomes the PR base; the gate's "integrated?" predicate goes async. IMP-129
  territory.

## 7. Decisions, Rationale & Alternatives

- **D1 — delivery-target-only scope (A).** `deliver_to` feeds the delivery axis;
  the ADR-006 D3 base resolver (`trunk_tree_ish`, env + ladder) is untouched.
  *Rationale:* matches IMP-124's literal scope; keeps base machinery sealed;
  PR-delivery is a delivery-axis concern, so entangling it with base resolution
  (alt B) would foreclose the very thing we want to vary.
  *Alt B (unify base+delivery into one trunk identity)* — rejected: reopens
  ADR-006 D3 + `freshest_descendant` semantics, bigger blast radius, entangles
  the PR axis. If IMP-129 wants one identity, that's a deliberate ADR-006
  amendment there.
- **D2 — bare-string `deliver_to`, default `refs/heads/main`.** Forward-compatible
  with PR delivery: same value is the merge target *and* the PR base; *mode* is an
  orthogonal future key. No structured config now (YAGNI); doc-comment names the
  extension point.
- **D3 — precedence:** explicit `--trunk` › `deliver_to` › default; `DOCTRINE_TRUNK_REF`
  env stays base-only (sha vs ref mismatch — can't FF a detached commit).
- **D4 — α hybrid (config defaults reads, not the write).** `--integrate`
  `--trunk`/`--edge` Options unchanged; close's write line sources the ref via the
  read verb. *Alt β (default `--integrate` trunk from config + `--no-trunk` for
  edge-only)* — rejected: changes `--integrate`'s contract, adds a flag, rewrites
  the edge-only tests (behaviour change needing re-blessing). α preserves the gate
  and gives the read verb a real consumer.
- **D5 — read verb `dispatch deliver-to`.** Thin stdout read; serves both close's
  write line and hand-driven git work (option b).

## 8. Risks & Mitigations

- **R1 — relaxing `--show-journal-trunk-oid requires="trunk"`** could regress the
  "must name a row" contract. *Mitigate:* resolve `--trunk` from config in the
  handler *before* dispatch; the verb still receives a concrete ref. Test both
  arms (with/without flag).
- **R2 — Rust `Default` vs serde-absent divergence** (the `""` trap). *Mitigate:*
  shared `default_deliver_to()` fn + hand-written `impl Default` + a parity unit
  test (mirrors the existing `empty_config_defaults_to_codex`).
- **R3 — edge-only regression.** *Mitigate:* `integrate()` untouched; the existing
  edge-only e2e suites are the proof and must stay green unchanged.
- **R4 — config-load duplication.** *Mitigate:* `load_doctrine_toml` extraction
  (DRY); no second loader.
- **R5 — `expect(dead_code)` on `DoctrineToml.dispatch` fires once read live.**
  *Mitigate:* remove the `#[cfg_attr(not(test), expect(dead_code, …))]` attribute
  (`dtoml.rs:38`) when the gate consumes the field. Caught in adversarial review.

## 9. Quality Engineering & Validation

- **Config (unit, `dispatch_config.rs`):** absent `deliver-to` → `refs/heads/main`;
  present → override; `DispatchConfig::default().deliver_to` parity with
  serde-absent (I1); combined with existing keys still parses.
- **Gate (`slice.rs`):** existing `trunk_integration` suites green **unchanged**
  (R3 proof); add a test that the gate honours a `deliver_to` override (config
  names a non-`main` ref → gate checks that ref).
- **CLI:** `dispatch deliver-to` prints the resolved ref (default + override);
  `--show-journal-trunk-oid` with no `--trunk` resolves from config, with
  `--trunk` honours the explicit value (I3); `--integrate --edge` with no
  `--trunk` still plans no trunk row (I2).
- **Prose:** `close/SKILL.md` *delivery* literals removed — line 74 (write,
  via verb), 95 (read, omit `--trunk` or verb), 96 (`git diff` compare, via verb),
  102/107 (explanatory + TODO). **Line 68 `--base refs/heads/main` stays** (fork
  base, concept #1, D1 boundary) — its presence is the proof base resolution was
  consciously left alone, not missed.

## 10. Review Notes

Adversarial self-review (internal pass) integrated:

- **F1 — `expect(dead_code)` trap.** Reading `doc.dispatch` live makes the
  `DoctrineToml.dispatch` dead-code expectation fire → compile error. → R5;
  remove the attr (`dtoml.rs:38`). Confirmed against source.
- **F2 — prose-check overreach.** Original §9 said "no `refs/heads/main` literal";
  `SKILL.md:68 --base` legitimately stays (fork base, out of scope). → §9 scoped
  to delivery literals; §2 records the full inventory + the D1 boundary.
- **F3 — `load_conduct` collateral.** It has a second caller + tests; kept as a
  delegating wrapper over `load_doctrine_toml`. → §5.3.
- **Confirmed facts:** `DoctrineToml: Default` (`dtoml.rs:18`); `run_status` has
  `root` in scope and already parses `doctrine.toml` once (`slice.rs:428`), so the
  gate read adds no second parse; `--trunk` already `Option<String>` in clap;
  edge-only `--integrate` is a live tested path (`e2e_dispatch_sync.rs:688`).

No governance conflict surfaced (ADR-001/006/012 all honoured; base resolver
sealed). Design ready for external/inquisition pass or `/plan`.
