# ISS-261: dispatch compares a filtered worktree oid against an unfiltered tree oid

Found while adversarially testing SL-232's DEC-089 (RV-314 round 4). **Pre-existing
and out of SL-232's scope** — recorded rather than fixed there, and DEC-089 does
*not* change it, because the neutralising flags deliberately do not join
`NORMATIVE_FLAGS`.

## The defect

`src/dispatch.rs:6122` reads:

```rust
if git::worktree_blob_oid(root, path)? != git::blob_oid_at(root, &stale, path)? {
```

The two sides are not comparable in a repository that uses gitattributes:

- `git::worktree_blob_oid` runs `git hash-object -- <path>`, which applies the
  **clean filter and eol/text conversion** to the worktree bytes.
- `git::blob_oid_at` runs `git ls-tree`, returning the **stored** blob oid, which
  is whatever was committed.

Its own doc comment states the contract it does not keep — "the blob oid the
**WORKING TREE** file at `path` would hash to … so verify can compare **worktree
bytes** against a baseline commit by oid". Under a `clean` filter it is comparing
the *filtered* projection of the worktree against the stored blob.

## Measured (git 2.54.0)

Under a `clean` filter that rewrites content to a constant, two files with
completely different contents hash identically through plain `hash-object`:

```
plain hash-object     a.dat=3c79cdb822b066786a19331faffc066a4543efb3
                      b.dat=3c79cdb822b066786a19331faffc066a4543efb3
--no-filters          a.dat=81920715936ccdb198cec402f62f990f5ec4838b
                      b.dat=d004ceeef7fd8d12daaca9febe3973d6767f1cba
```

And under `text eol=crlf`, plain `hash-object` returns the **LF-normalised** oid
(`0eabd516…`) rather than the raw CRLF bytes (`126799cc…`); `--no-filters`
returns the raw oid.

So the comparison can read *equal* when the bytes differ (both sides collapse to
the filter's output) and *unequal* when they do not (eol conversion). Which
direction bites depends on the attribute in play.

## Remedy

`git hash-object --no-filters -- <path>`. Measured sufficient for **both** the
clean-filter and the eol routes, and it needs no derived empty-tree oid, so it
carries no git version floor (unlike `--attr-source`, which is git 2.40).

Consider whether `worktree_blob_oid`'s doc comment should also state the raw-byte
contract explicitly, since the whole class of defect here is a helper whose name
and comment promise bytes while its implementation delivers git's converted view.

## Live population

**0 in this repository** — it has no `.gitattributes`, no
`$GIT_COMMON_DIR/info/attributes`, and `core.attributesFile` unset. POL-002 makes
a client project using attributes a real case.

## Related

- SL-232 / RV-314 F-19, F-21 — the same class at the memory-verify probes.
- DEC-089 — fixes the sibling instance in `capture()`'s `untracked_fingerprint`,
  which *is* in SL-232's scope because that slice changes `capture()`.
- ISS-262 — the other pre-existing git-layer defect found in the same sweep.

## Resolved 2026-07-29 (light path, with ISS-262)

`worktree_blob_oid` now runs `hash-object --no-filters`, and its doc comment
states the raw-byte contract. One production caller (`forward_sync`'s per-path
worktree leg in `src/dispatch.rs` — the item cites `:6122`, actually `:6164` by
the time of the fix, cf. IMP-344).

**The remedy above understated the tradeoff, and the correction is recorded here
rather than smoothed.** `--no-filters` is not free in a repository that opted into
conversion: the worktree side then reads raw bytes while the tree side holds the
converted blob, so an *untouched* path reads as edited and the clobber guard
refuses. Two tests pin both halves deliberately — the lossy-`clean` collapse
(distinct contents → one oid `e96ee3ab…`, the silent-overwrite hazard) and the
`eol=crlf` divergence (fail-closed noise, preferred), plus a call-site comment
telling the next reader not to revert it.

Measured on git **2.55.0** (the tickets say 2.54.0; the jail moved): conversion
requires an explicit opt-in — committed `text`/`eol`/`filter` attributes *or*
`core.autocrlf`. With neither set, filtered and `--no-filters` oids are
byte-identical, so the flag is a no-op on the ordinary repository and the general
case is unaffected. `core.autocrlf` is the likelier real-world route (machine-local
config, not committed attributes). Windows support is a declared non-goal.
