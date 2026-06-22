# IMP-152: Document `review_prime` domain_map TOML format

**Surface:** `src/review.rs` L2544 (`PrimeArgs`), `run_prime` — consumed by `review_prime --from <file>` or stdin.  
**MCP schema:** `src/mcp_server/tools.rs` — `from` parameter says "Read the curated domain_map from a file (default: stdin)" with zero format guidance.

## The gap

The `domain_map` is the review's curated context model — areas, tracked paths,
invariants, and risks. It is the protocol's "priming" step (`review-ledger.md` §2).
Every agent doing a review must construct one. But the expected TOML format is
undocumented: the walkthrough agent hit 4 errors before getting a valid map:

1. JSON rejected → expects TOML
2. `[areas.src]` rejected → needs `[[area]]`
3. `tracked = [...]` rejected → field is `paths`
4. `invariants`/`risks` under `[[area]]` silently ignored → need `[[invariant]]`/`[[risk]]`

## Design questions

The user wants this as a **shipped memory** (not inline in MCP descriptions) so
it's referenceable from skills and MCP help without bloating context. Two concerns:

a) **Staleness.** If the domain_map format changes, how does the memory stay
   in sync? Options: a test that validates the documented format against the
   actual parser, or embedding a format version/schema.

b) **Scope.** What other opaque TOML formats deserve similar treatment?
   Could this be a general "format registry" pattern.

## Discovered by

IMP-150 walkthrough audit.
