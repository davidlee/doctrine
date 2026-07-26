# ISS-248: selector doctor redundancy scan is class-blind — a design-target flagged redundant against a scope-relevant peer

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`doctrine slice selector doctor 230` reports:

```
SL-230: 1 selector finding(s)
  redundant     tests/e2e_mcp_server.rs  (subsumed by tests/**)
```

SL-230's selector set is:

| selector | intent |
|---|---|
| `src/entity.rs` | design-target |
| `src/memory.rs` | design-target |
| `src/mcp_server/tools.rs` | design-target |
| `tests/e2e_mcp_server.rs` | **design-target** |
| `tests/**` | **scope-relevant** |
| `.doctrine/spec/tech/007/**` | scope-relevant |

The two selectors named in the finding are in **different classes**.

## Why it is wrong

`slice conformance` cross-checks recorded source deltas against **design-target
selectors only** (its own help text: *"Cross-check a slice's recorded source
deltas against its design-target selectors"*). A `scope-relevant` selector
declares relevance, not an intended edit, and does not authorise one.

So the two selectors are not substitutes: `tests/**` matching a superset of
`tests/e2e_mcp_server.rs` does **not** mean the latter is redundant. Acting on
the advisory — deleting the design-target — would have converted every edit to
that file into an **undeclared edit** at audit. On SL-230 that is PHASE-05's
entire deliverable.

The advisory is therefore worse than noise: it is confidently wrong in the
direction that costs an audit finding, and it is the one check the pre-execution
readiness moment is designed to lean on.

## Cause

`run_selector_doctor` (`src/slice.rs:2757-2771`) builds the peer set from every
selector regardless of intent, then passes that flat set as `others` to each
`diagnose_selector` call:

```rust
let mut peers: Vec<String> = doc.selectors.iter().map(|s| s.selector.clone()).collect();
peers.sort();
peers.dedup();
let peer_refs: Vec<&str> = peers.iter().map(String::as_str).collect();

for sel in &doc.selectors {
    findings.extend(conformance::diagnose_selector(
        &sel.selector,
        selector_scope(sel.intent),   // <- own class is passed …
        &universe,
        &peer_refs,                   // <- … but peers are class-flattened
    ));
}
```

The pure predicate `conformance::diagnose_selector` (`src/conformance.rs:255`)
already receives the subject's own `SelectorScope`, so the class information
exists at the call site and is discarded for the peer set only. The redundancy
arm (`src/conformance.rs:285`, *"a peer whose match set is a PROPER superset of
ours absorbs us"*) is correct in isolation — subsumption is only meaningful
**within** a class.

## Suggested fix

Partition `peers` by `selector_scope(intent)` and pass each selector only its
same-class peers. Small and local; the pure predicate needs no change.

Worth deciding at the same time whether the `broad` arm has the same latitude
problem — a `scope-relevant` glob like `tests/**` is *supposed* to be broad, so
a breadth warning may be right for design-targets and wrong for scope-relevant
selectors. Same class-blindness, different arm.

## Coverage

`diagnose_redundant_when_subsumed_by_a_broader_peer`
(`src/conformance.rs:353`) exercises the pure predicate with an implicitly
single-class peer set, so it passes and always would — the defect lives in the
shell that assembles `others`, which has no test asserting cross-class
independence. A regression test belongs at the `run_selector_doctor` /
`render_selector_doctor` seam.

## Provenance

Surfaced by the SL-230 pre-execution readiness pass (2026-07-26). The advisory
was refused there and SL-230's selector set left unchanged; no slice state
depends on this being fixed. Also recorded as an RFC-011 token-efficiency case
note — establishing that the advisory was unsafe to follow cost ~4 tool calls.
