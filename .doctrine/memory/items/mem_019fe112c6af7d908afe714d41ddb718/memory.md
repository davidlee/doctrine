# Adding a `Category` to the doctor: six production sites, and one loses data

`src/finding.rs` is not a single-source enum. A new `Category` variant must be
added at **six** production sites plus two tests. Five are ordinary; the sixth is
a silent-loss bug if you miss it.

1. `enum Category` — the variant.
2. `severity()` — a match arm. **Exhaustive**, so it will not compile without you.
3. `ordinal()` — a match arm. Exhaustive.
4. `display_name()` — a match arm, plus a `CATEGORY_NAME_*` const beside the
   others (STD-001). Exhaustive.
5. `CATEGORIES_BY_ORDINAL: [Category; N]` — the array **and its length**.
6. **`render_findings`'s bucket array.** This is the one to know about.

## Why #6 is different

`render_findings` buckets findings by ordinal:

    let mut by_category: [Vec<&Finding>; N] = [ … ];
    let idx = usize::from(f.category.ordinal());
    if let Some(bucket) = by_category.get_mut(idx) { bucket.push(f); }

`get_mut` on an out-of-range index returns `None`, so a category whose ordinal
exceeds the array's hand-written length is **dropped from the render AND from the
finding total** — no panic, no lint, no compile error. The check runs, finds real
problems, and prints nothing. *A doctor check reporting nothing is
indistinguishable from a clean corpus.*

**As of SL-249 PHASE-03 this site is fixed by construction** — the array is now
sized `[Vec<&Finding>; CATEGORIES_BY_ORDINAL.len()]` via
`[const { Vec::new() }; CATEGORIES_BY_ORDINAL.len()]`, so it can no longer fall
behind. Kept in this memory because the *shape* recurs: any hand-written length
mirroring an enum is this bug waiting, and `get`/`get_mut` is what turns it from
a panic into silence. Prefer deriving the length; if you cannot, prefer indexing
that panics over indexing that shrugs.

## The two tests, and which one saves you

- `test_severity_mapping` (`src/finding.rs`) is **hand-enumerated** and will NOT
  fail when you forget a variant. Add the row yourself.
- `test_render_all_categories` **will** fail — it builds one finding per
  `CATEGORIES_BY_ORDINAL` entry, renders, and asserts each `display_name` appears.
  It is the canary that catches #5 and #6 together. Treat a failure here as
  "something downstream drops my category", not as a test to adjust.

Then wire the check itself in `src/commands/doctor.rs` (`run_doctor`, a numbered
`findings.extend(..)`) and update that module's doc comment, which states the
check count and lists them by name — prose that must move with the code.

Measured at SL-249 PHASE-03 (`Category::InertFacetKey`, check #12). Compare
[[mem.pattern.doctrine.record-kind-touch-sites]] — same class of hazard for
record kinds.
