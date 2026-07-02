# IMP-235: verify-vt patterns are line-anchored: formatter-wrapped multi-line asserts false-Fail

## Context

Surfaced by the SL-180 dispatch drive (PHASE-01 VT-2). `slice verify-vt` matches a
VT's `patterns` (regex) mandate **per line**, not over the whole file:

```rust
// src/vtgate.rs:139
if !source.lines().any(|line| re.is_match(line)) { ... Fail }
```

This is deliberate — line-anchoring is load-bearing for the stronger `^`-anchored
shape the module advertises (`^\s*fn check_vt`, `^\s*assert_eq!\(.*census`): those
anchors only mean anything against a single line. See the module rustdoc (§ line 21,
33) and the `patterns` escalation tests (line 406).

The sharp edge: a **correctly-authored** assertion whose token the mandate matches
can still false-Fail if the formatter wraps the statement across lines. On SL-180
VT-2 mandated `patterns = ["assert.*café"]`; the test was correct, but rustfmt
wrapped the 2-arg `assert!(cond, "…café…")` so `assert!(` and `"café"` landed on
different lines — the line-anchored regex matched neither. The gate reported Fail on
a passing test, costing a reconcile round. Fix that landed: collapse the assertion
to a single-arg form (`assert!(actual.contains_key("src/café.rs"))`) so the token
survives on the `assert` line.

## Why this is a real gap, not just operator error

The VT mandate is authored **before** the test exists, and the author cannot predict
where rustfmt will wrap the eventual statement. So "just keep it on one line" is a
convention the mandate can't enforce and the author can't verify at authoring time —
the failure only appears at conclude/verify, in the coord tree, mid-dispatch.

## Options (not yet chosen)

1. **Authoring guidance** (cheapest): document in `/plan` that `patterns` are
   line-anchored — mandate a token that lands on one line (a symbol, a call head, a
   single-arg assert), never a `head … tail` span the formatter may break. Pairs
   with IMP-209 (structured VT authoring).
2. **Formatter-normalised match**: run the same match over a whitespace-collapsed
   view *in addition to* the per-line pass, so a wrapped statement still matches —
   without losing `^`-anchored semantics (keep both passes; `patterns` that start
   with `^`/`$` stay line-only). Risk: false-PASS if the collapsed token spans
   unrelated statements. Needs care.
3. **`keywords` instead of `patterns`** for span-y mandates: `keywords` is a
   whole-file substring (`source.contains`), so it already crosses line wraps —
   but it can't express the `assert.*X` co-occurrence the author wanted. A
   multi-keyword AND (`["assert", "café"]`) is close but doesn't bind them to the
   same statement.

## Neighbours

- IMP-228 (resolved) — the **inverse** blind spot: keyword pre-exists in prod →
  false **PASS**. This item is false **FAIL**. Same gate, opposite direction.
- IMP-209 (open) — `/plan` should author structured VT mandates so verify-vt has
  signal; option 1 above is naturally a rider on that skill change.
- Source: SL-180 PHASE-01 VT-2 reconcile; RFC-011 case notes.
