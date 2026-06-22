# IMP-151: `review_prime` seed field — JSON Schema vs Rust serde mismatch

**Surface:** `src/review.rs` L2547 — `PrimeArgs { seed: bool }`  
**MCP schema:** `src/mcp_server/tools.rs` — `required: ["reference"]` (seed NOT listed)

## The mismatch

The MCP JSON Schema advertises `seed` as optional (only `reference` is required).
But the Rust struct has `seed: bool` (not `Option<bool>`) with no `#[serde(default)]`.
Calling the tool without `seed` fails with:

```
missing field `seed`
```

An MCP consumer that trusts the schema will omit `seed` and get an error.

## Fix

Either:
- Make `seed: Option<bool>` with `#[serde(default)]` (defaults to `false` = normal prime mode), or
- Add `"seed"` to the `required` array in the schema

The first is preferred — `seed=false` is the sensible default (normal prime).

## Discovered by

IMP-150 walkthrough audit — agent exercising all review MCP tools end-to-end.
