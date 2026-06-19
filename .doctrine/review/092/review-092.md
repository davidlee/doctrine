# Review RV-092 — design of SL-109

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisition interrogates SL-109's design.md against ADR-007 (lock/baton/CAS),
ADR-011 (harness-agnostic interface), the storage rule, the "no parallel
implementation" mandate, and the design's own claims. Eleven claims were named in
the handover; nine were specifically assigned as lines of attack.

## Synthesis

### Summary Judgement

**The design is mostly sound** — one substantive flaw (error mapping by
string-parsing, F-1), four specification gaps, and zero architecture-level
heresies. No blockers. The "no parallel implementation" claim holds: MCP handlers
truly are thin wrappers over the same `run_*` functions that the CLI calls. The
ADR-007 lock/baton/CAS protocol is honoured by construction — the MCP path rides
`with_turn` exactly as the CLI does. ADR-011's harness-agnostic interface pattern
is followed: one contract (the review engine), delivered through a new transport
(MCP/stdio).

### Findings Disposition

| Finding | Severity | Charge | Verdict |
|---------|----------|--------|---------|
| F-1 | major | Error mapping from `anyhow::Error` by string-parsing is brittle and untestable | **fix-now** — add a structured `ReviewError` enum before implementation |
| F-2 | minor | `Showed` variant computes both JSON and formatted unconditionally | **fix-now** — carry structured data; let consumers format |
| F-3 | minor | `cache_verdict: Option<String>` in Status variant is ambiguous | **fix-now** — use structured fields (`cache_primed: bool`, `stale_paths: Vec<String>`) |
| F-4 | minor | `print_review()` output contract is unspecified | **fix-now** — add concrete output contract table to design.md |
| F-5 | minor | `Primed` variant carries semantically confused `stale: Vec<String>` | **fix-now** — remove `stale`; rename `cache_paths` to `tracked_paths` |

### Claims That Held

The following design claims survived adversarial interrogation:

1. **`ReviewOutput` enum correctness (claim 1).** The 11-variant enum correctly
   separates structured from formatted data. Minor concerns (F-2, F-3, F-5) are
   field-level, not structural.

2. **`with_turn` generic over closure return `T` (claim 2).** Sound and
   well-reasoned. Only `run_raise` needs `T != ()`; all other verbs stay `T = ()`.
   No turbofish or inference issues.

3. **Hand-rolled MCP protocol (claim 3).** The tools-only surface (initialize,
   tools/list, tools/call) is ~300 lines. `tokio` with `io-util` is already in
   `Cargo.toml`. Zero new crate deps confirmed.

4. **No parallel implementation (claim 6).** MCP handlers call `run_*`, not
   engine internals. The lock/baton/CAS protocol is not reimplemented.

5. **ADR-007 lock/baton honoured (claim 7).** The MCP path goes through
   `with_turn` which acquires the lock, checks CAS, writes authored-first
   baton-last. Lock contention under concurrent MCP calls is identical to
   concurrent CLI invocations (RSK-003).

6. **Zero test impact (claim 8).** Tests call `run_*` with `.unwrap()` — the
   new `ReviewOutput` return is silently discarded. Error-path tests use
   `.unwrap_err()` — unaffected by the success-type change.

### Standing Risks

- **RSK-002 (`Deserialize` on `Severity`/`Facet`).** The design acknowledges
  this and proposes `#[serde(deserialize_with)]` bridges to existing `parse`
  methods. Low risk, correctly mitigated.
- **RSK-003 (Baton CAS under batch mutation).** Lock serializes concurrent
  writes per ADR-007 D-C4a. Verified in execute phase via VH-5 agent test run.
- **MCP protocol edge cases (VH-3/VH-4).** Hand-rolled JSON-RPC framing may
  have subtle bugs (notification handling, empty arguments, `id` field
  round-trip). Integration tests will catch these; not a design defect.

### Tolerated Tradeoffs

None consciously tolerated. All findings are `fix-now`.

### Ordered Penance

The design author shall, before the plan phase:

1. **Add `ReviewError` enum** (F-1). Replace `anyhow::bail!` in verb handlers
   with structured error returns. The MCP error mapper matches on enum variants.

2. **Restructure `Showed` variant** (F-2). Carry structured data; consumers
   format it. Eliminate dual-computation.

3. **Fix `Status.cache_verdict`** (F-3). Use `cache_primed: bool` and
   `stale_paths: Vec<String>`.

4. **Add output contract table** (F-4). Map each `ReviewOutput` variant to its
   expected stdout text in `design.md`.

5. **Fix `Primed` variant** (F-5). Remove `stale`; use `tracked_paths` +
   `areas_count`.

---

**HERESIS URITOR; DOCTRINA MANET**
