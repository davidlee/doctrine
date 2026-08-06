# DEC-176: Kind coverage in governance is pinned by a canary

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The failure this guards against

Twice now, a slice has added something and left its governance behind, and
nothing noticed.

`SL-222` PHASE-09's objective read *"facet_write [value]/[estimate] machinery
deletes (risk/tags survive)"*. Its exit criteria checked a grep-gate, a tripwire
suite, and a green build with a baseline diff. None of them checked for the
deletion. It never happened; the audit passed; `RV-284`'s nine findings did not
include it; no backlog item owns it fourteen months later.

`SL-159` scoped a *"Governance axis — routes through a Revision (ADR-013): cut
after design, settle in reconciliation"* and wrote no criterion for it. No
revision in the corpus amends `SPEC-019` or `PRD-010` on the kind set.
`SL-197` added `CPT` with no governance axis at all.

Both are failures of **omission**. That is what makes prose criteria
insufficient here rather than merely weaker: an agent verifying *"SPEC-019
enumerates seven kinds"* checks the claim it was handed. Neither prior failure
had a claim to check.

## The canary

```rust
#[test]
fn every_record_kind_is_named_in_spec_019() {
    let text = fs::read_to_string(
        test_support::repo_root().join(".doctrine/spec/tech/019/spec-019.md"),
    ).expect("SPEC-019 body");
    for prefix in kinds::RECORD {
        assert!(text.contains(prefix), "SPEC-019 does not name record kind {prefix}");
    }
}
```

Both seams exist already:

- `src/test_support.rs::repo_root()` — a **runtime** repo-root resolver. It
  exists because the jail shares one `CARGO_TARGET_DIR` across worktrees, so a
  compile-time `env!("CARGO_MANIFEST_DIR")` bakes the wrong tree's path;
  `tests/e2e_no_baked_paths.rs` bans the macro outright.
- `src/kinds/mod.rs:57` — `pub(crate) const RECORD: &[&str] = &[ASM, DEC, QUE,
  CON, EVD, HYP, CPT]`, the canonical prefix list.

Both are `pub(crate)` and the crate has no `[lib]` target, so this is an
in-crate unit test, not one under `tests/`. That is the entirety of the
complication.

## The wrinkle to handle at write time

A bare `contains` on a three-character prefix can pass falsely — `"CON"` matches
inside any uppercase token containing it. Assert on the paired form the specs
already use, `` assumption (`ASM`) ``, so the check means what it reads as.

## Placement, and POL-002

This lives in doctrine's own test suite as dogfooding. *"Every record kind must
be named in a spec"* is a host-project convention, and POL-002 forbids the
platform carrying it — so this must not later migrate into `doctrine validate`.
The rule is doctrine-the-project's, not doctrine-the-engine's.

## Note on reading the live corpus

No existing test asserts over the repo's own `.doctrine/`; the spec-reading
tests all build temp fixtures. This one reads a single fixed file rather than
scanning a tree, which keeps it clear of the worktree fragility `ISS-024`
records for corpus scanners.
