# IMP-325: Discriminate verified_sha's kind on the record: commit anchor vs checkout-state stamp

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Routed out of **SL-232**'s design round by **DEC-055**. Not built in SL-232:
objective 7 emits one flat *cannot determine* finding for all three causes.

## The defect

`stamp_verification` (`src/memory.rs::stamp_verification`, the branch at
`:3465-3470`) writes **two incompatible value kinds** into one field:

```rust
let verification_value = if allow_dirty && frame.anchor_kind == AnchorKind::CheckoutState {
    frame.checkout_state_id.as_str()   // sha256 of the dirty checkout — NOT a commit
} else {
    frame.commit.as_str()              // a real commit id
};
git.insert("verified_sha", toml_edit::value(verification_value));
```

Nothing on the record says which. `[git].anchor_kind` does **not** serve: it
describes the *born* frame at `record` time, not the verify-time stamp (389
memories carry 328 `checkout_state` / 60 `commit` anchors, against 59 attested).

Every consumer passes `verified_sha` to `git::commits_touching` as `since`. The
ancestry guard (`src/git.rs:2493`) contains the damage — a non-commit fails
`merge-base --is-ancestor` and folds to `None` — so this is **latent, not live
over-trust**. It costs reach, not correctness.

## Measured cost

At HEAD `377022dfa`, all 59 attested memories
(`.doctrine/slice/232/probes/populations.py`):

| `verified_sha` | guard | count |
|---|---|---|
| 40-hex commit, ancestor | ok | 25 |
| 40-hex commit, non-ancestor | exit 1 | 8 |
| 40-hex, not an object | exit 128 | 2 |
| **64-hex `checkout_state_id`** | exit 128 | **24** |

**24 of 59 attestations (41%) were never commit-anchored.** They cannot be
staleness-checked and never will be, until re-verified. Under SL-232 objective 7
they render as *cannot determine drift*, which is true but does not name the
remedy (re-verify) or the fact that no clone or fetch will ever resolve them.

## Do not implement this by inspecting the stamp

**Falsified during SL-232's design round** — see DEC-055. Discriminating by the
stamp's width (64 hex ⇒ not a commit) is wrong: `git init --object-format=sha256`
yields **64-hex commit ids**, so on a sha256 repo the rule misclassifies every
genuine non-ancestor commit. It also fails to catch the 2 dangling 40-hex rows
without `cat-file -e`, which is the ref-set-dependent instrument RV-307 **F-31**
already refuted. And doctrine has **no** sha-width assumption anywhere in `src/`
today; this would be the first.

The correct shape is a **discriminator written at stamp time** — the writer knows
the kind, so the reader must not re-derive it from local repository state. That
is the same move as DEC-053 (emit by field of origin) and SL-232 objective 3 (the
declared boundary).

## Sequencing

Groups with **IMP-318** (persist attested coverage) and **QUE-173** (body digest)
under **OQ-A**'s ruling: all three are *machine-written outputs of a verify run*,
distinct in writer and lifecycle from objective 3's *authored input*. Sharing a
TOML file is not sharing a change.

Consider together with the wider question of whether `--allow-dirty` should stamp
`verified_sha` at all, versus a separate field — SL-232 preserves the current
semantics unchanged (invariant I4), so that is not settled here.
