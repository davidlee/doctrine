# ISS-501: ZERO_OID is a hardcoded sha1 constant, wrong on sha256 repos

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found by RV-411 (CHR-172 test-quality review). Sibling of ISS-262, which fixed
only the empty-tree constant.

## Defect

`git::ZERO_OID` (`src/git.rs`) is 40 zeros — the sha1 null object id. On a
sha256 repository git refuses it:

```
$ git init --object-format=sha256 r && … && git update-ref refs/x HEAD 0000…(40)
fatal: 0000000000000000000000000000000000000000: not a valid old SHA1
```

Production users of `ZERO_OID` as a CAS (compare-and-swap) "must not exist"
expectation: `src/reserve.rs` (`push_ref_cas`, `update_ref_cas`) and
`src/dispatch.rs` (ref creation / comparison sites). Reservation and dispatch
ref creation therefore fail on sha256 repos.

## Why tests missed it

`git::tests` hardcode sha1 widths (`len() == 40`, `"0".repeat(40)`), and the CAS
and push tests run only on the default object format. Running the module with
`GIT_DEFAULT_HASH=sha256` fails 10 tests.

## Fix

Derive the null oid per repo (as `empty_tree_oid` now does, ISS-262), assert
oid width by object format, and run the CAS/push tests on both formats
(`with_object_format` already exists).
