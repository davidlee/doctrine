# ISS-262: EMPTY_TREE_OID is a hardcoded sha1 constant, wrong on sha256 repos

Found while adversarially testing SL-232's DEC-089 (RV-314 round 4). **Pre-existing
and out of SL-232's scope.**

## The defect

```rust
// src/git.rs
pub(crate) const EMPTY_TREE_OID: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
```

That is the **sha1** empty-tree oid. It is used by the reservation subsystem —
`commit-tree EMPTY_TREE_OID` for the dangling reservation commit — and by
`diff_doctrine_paths` as the absent-side operand.

RV-314 F-24 raised exactly this hazard against SL-232's own proposed use of an
empty-tree oid, and the design answered it by **deriving** the value per
repository. The same argument applies to the constant that already exists, which
nobody had looked at.

## Measured (git 2.54.0)

```
git init --object-format=sha1   → git hash-object -t tree /dev/null
                                  = 4b825dc642cb6eb9a060e54bf8d69288fbee4904
git init --object-format=sha256 → git hash-object -t tree /dev/null
                                  = 6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321
```

Passing the sha1 oid to a sha256 repository is `fatal: bad object`, exit 128 —
so this fails **loudly** rather than silently, which is why it is an issue rather
than a risk. A doctrine repo created with `--object-format=sha256` cannot take a
reservation.

## Remedy

Derive it the same way DEC-089 does — `git hash-object -t tree /dev/null` run
**inside the target repository** (measured: run outside any repository it returns
the sha1 value unconditionally, regardless of the target repo's algorithm, and it
writes no object so it stays read-only).

Note STD-001 also bites here: the literal is a magic string standing in for a
derived fact.

## Live population

**0** — doctrine repos are sha1 today. This is latent, not active.

## Related

- SL-232 / RV-314 F-24 — the finding that surfaced the class.
- DEC-089 — derives the oid per repository for the SL-232 use.
- DEC-055 / IMP-325 — the other place this repo has had to reason about sha256
  object-format width (`verified_sha`'s 64-hex commit ids), where a
  width-discriminating shortcut was falsified for the same reason.
- ISS-261 — the other pre-existing git-layer defect found in the same sweep.

## Resolved 2026-07-29 (light path, with ISS-261)

`git::EMPTY_TREE_OID` is gone; `git::empty_tree_oid(root) -> Result<String>`
replaces it, deriving per repository. Two production sites, both already holding
`root` and both cheap (once per reservation; unrelated-histories-only for the
`merge-base` fallback), so no memoisation: `commit_empty_tree_as` (`src/git.rs`)
and `corpus_clobber_refusal` (`src/dispatch.rs`).

Derivation is `hash-object -t tree --stdin` over empty stdin rather than DEC-089's
`/dev/null` — same input, but it rides the existing `git_stdin` wrapper, so it
stays on the one `NORMATIVE_FLAGS` chokepoint (EX-1) and adds no second runner or
unix-only path. No `-w`.

Four tests. The load-bearing one is the regression: a reservation `commit_empty_tree_as`
on a `--object-format=sha256` `ScratchRepo`, which went red exactly as predicted —
`commit-tree empty-tree: fatal: not a valid object name 4b825dc6…` — and green on
the derivation. Its emptiness assertion counts `ls-tree` *entries* rather than
comparing an oid literal, since the oid is per-format and the emptiness is not. The
other three pin: the sha1 value (so swapping a literal for a git call moves nothing
where the literal was right), the sha256 value (`6ef19b41…`, matching DEC-089's
measurement), and `count-objects` unchanged across the call (read-only, I2).

`ScratchRepo::new()` now delegates to `with_object_format(Option<&str>)`.

Measured on git 2.55.0. STD-001 satisfied: the derived fact is no longer a literal.
