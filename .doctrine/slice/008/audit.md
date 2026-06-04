# SL-008 audit — memory retrieval (find/retrieve + scope ranking + staleness)

Hand-authored close-out (no `slice audit` scaffold yet — known CLI gap). Verifies
the shipped reader against `design.md` and the locked decisions D1–D19 / B1–B20,
and records the durable risks, divergences, and follow-ups harvested from the five
phase sheets.

- **Status:** all 5 phases completed; `just check` = 303 unit + 4 e2e green, clippy
  zero (bin). Commits `5a826c2` (01) · `376c573` (02) · `0b3c9f8` (03) · `76aaaf0`
  (04) · `79dd927` (05) · `6b43858` (05 DRY refactor).
- **Verdict:** ships. Two intentional design divergences (D-A1, D-A2 below) need a
  design fold-in; one edge (A-1) is a recorded accepted-risk, not a defect.

## Coverage vs design

| Area | Design | Shipped | Note |
|------|--------|---------|------|
| Pure predicate layer | §5.5 B1/B4/B9/B20, D6/D15 | `match_scope`/`base_filter`/`thread_expiry` | filters DROP, never `Ord` keys (B1) ✓ |
| Ordering core | §5.2 8-key table, D5/D12/D18 | `rank` 9-elem tuple (key 9 = uid,key) | total order; shuffle-invariant property test ✓ |
| Staleness | §5.5 4-branch, D11 | `staleness` attestation-keyed | `GitFacts` by value; target-None ⇒ branch-1 Unknown (notes §02 divergence) |
| Git seam | §5.2, D9/D10, B17/B18 | `commits_touching` + ancestry precheck | non-ancestor ⇒ None (no over-count) ✓ |
| Shared shell | §5.1/§5.4, B18/B19 | `freeze`/`query`/`load_query` | one capture + clock; degrade-not-fail ✓ |
| find surface | §5.2, D8/D17, B8 | `format_find`, full-uid rows | holdback-EXEMPT; trust+sev columns visible ✓ |
| retrieve surface | §5.1/§5.4, D2/D8/D17/D19, B7/B10 | `run_retrieve` | per-block nonce, holdback, limit, staleness header ✓ |

## Security findings (the retrieve crux)

- **D2 per-block nonce — VERIFIED.** A fresh `uuid::v4` is minted INSIDE the render
  loop (one per hit); the e2e parses the `=== END MEMORY <nonce> ===` fences and
  asserts N blocks ⇒ N distinct nonces. One nonce across N bodies would let body *i*
  forge body *i+1*'s close — the trap is closed.
- **B7/D8 holdback — VERIFIED non-bypassable.** `low ∧ severity≥high` is suppressed
  PRE-render via the pure `select_shown` seam: a held-back memory's body is never
  read (`read_body` not called) and never framed. No `--include-held-back` flag
  exists. `--min-trust` only RAISES the floor (`min`-clamp). The find/retrieve
  asymmetry (find shows the risky memory, retrieve omits it) is e2e-proven.
- **F-A2 header injection — still defended.** The new `staleness:` line is doctrine-
  computed (not free text), and all free header values remain `scrub_line`d; the
  existing F-A2 newline-forge tests pass unchanged.

## Divergences to fold into design

- **D-A1 — suppress-then-take (K3), ratified by the user.** §5.1's pseudocode reads
  `take(limit, [Ranked])` then render with "pre-render" suppression — literally
  take-then-suppress, which lets a top-ranked held-back memory shrink the shown
  block count below `--limit`. Shipped behaviour filters held-back across ALL ranked
  survivors THEN `take(limit)`, so the agent-context budget is filled with shown
  memories and a held-back memory never steals a slot. Security invariant holds
  under either order. → update §5.1 pseudocode to `take(limit, suppress([Ranked]))`.
- **D-A2 — `--min-trust` = minimum-trust-to-PASS (K4), ratified.** Design states
  "raises the floor to L" without pinning the predicate direction. Shipped: floor is
  a `trust_rank`, default `medium`; `held_back = severity∈{critical,high} ∧
  trust_rank(m) > floor`. `--min-trust high` ⇒ low+medium held, HIGH passes (not
  "all high-severity suppressed"). Matches the flag name. → record the predicate in
  §5.2.

## Accepted risks / known edges

- **A-1 — `severity_rank` "unknown ⇒ worst bucket" means a malformed severity
  escapes the holdback.** `held_back` reuses the ranking ordinal where unknown/empty
  severity sorts as rank 5 (LEAST severe), so `≤ high` is false and the memory is
  NOT held. Acceptable in v1: severity is a doctrine-authored axis (the store is
  tool-authored, not attacker-controlled — the same trust assumption that lets a
  memory self-assert `trust_level`). If the store ever admits untrusted severity,
  the holdback gate should fail safe (unknown ⇒ treat as high). Recorded, not fixed.
- **A-2 — capture failure degrades silently** (B18/B19, notes §04): a
  multi-root/submodule tree yields `target=None` ⇒ staleness `Unknown`/time-mode,
  hiding genuine git breakage. Accepted under D19 (staleness stays visible); a future
  `--explain`/warn could surface it.
- **A-3 — `record` has no `--trust`/`--severity` flag**, so the holdback is only
  exercisable end-to-end by post-editing a `memory.toml` (e2e `make_risky`). Not a
  reader defect — a producer-surface gap (SL-007 scope). Flag if a future slice wants
  CLI-settable risk axes.

## Doctrine adherence

- Pure/impure split honoured: the pure core (`retrieve.rs` ≤ `rank`) reads no clock,
  git, or disk; `Loaded`/`freeze`/`run_*` are the thin shell. `held_back`/
  `holdback_floor`/`select_shown` are pure and unit-tested without fs/stdout.
- No parallel impl: the find/retrieve prologue was DRYed into `load_query`
  (`6b43858`) after the code-review flagged the copy-paste; `render_show` was reused
  (not forked into a batch renderer — D2/EX-1); `read_body` rides the `safe_join` H1
  chokepoint.
- Behaviour-preservation gate: the SL-005 `show` goldens stayed green through the
  `render_show` arity change (None ⇒ byte-identical) — the proof the shared renderer
  was not disturbed.

## second-pass: confirmed (independent /code-review, 2026-06-05)

Adversarial second pass over `HEAD~3..HEAD` (PHASE-05 + DRY refactor + close-out).
Tried to REFUTE the four crux claims; all hold. Gate re-run green: 303 unit + 4 e2e,
clippy zero (bin).

- **D2 nonce** — `retrieve.rs:783` mints inside the `for c in select_shown(…)` loop,
  never hoisted; e2e asserts N blocks ⇒ N distinct close fences. Confirmed.
- **B7/D8 holdback** — `select_shown` suppresses `held_back` PRE-`take`; `read_body`
  runs only over the ≤limit shown set (`:780`). No `--include-held-back`. Traced all
  three `--min-trust` tiers through `holdback_floor`'s `.min(default)` clamp: `low`
  is a no-op, `high` raises — non-bypassable downward. Confirmed.
- **A-1 (the live edge)** — independently agree with DEFER. The holdback already
  trusts the store on the `trust_level` axis (a hostile author escapes via
  `trust=high` regardless), so fail-safing only severity (`unknown ⇒ high`) closes a
  strictly smaller hole than the trust axis leaves open — it buys nothing until the
  store admits untrusted severity. Accepted v1 risk, not a defect; no re-open.
- **render_show arity** — `:882` `map_or(String::new(), …)` ⇒ `None` is byte-identical;
  SL-005 goldens + the with/without unit tests prove it. Confirmed.

Independent checks beyond the four probes: body reads bounded by `limit ≤ 20` (no
unbounded read); the `staleness:` header is a doctrine-computed enum label (F-A2
injection-safe); the §10 design fold-in honestly records D-A1/D-A2/A-1 rather than
retrofitting the old pseudocode. No defect found — close-out stands.
