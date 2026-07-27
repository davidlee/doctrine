# DEC-055: validate reports one undeterminable state; verified_sha's kind is not discriminated here

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

Objective 7's tri-state is **flat**: `commits_touching` returning `None` yields
one finding — *cannot determine drift* — regardless of which of the three live
causes produced it. `validate` does **not** classify the cause.

The measured cause distribution is recorded below and written into the design as
a risk, but it drives no branch in the code. Discriminating `verified_sha`'s kind
is routed as **IMP-325**, sequenced with IMP-318 under OQ-A's existing ruling
(machine-written outputs of a verify run are not this slice's authored input).

Taken by the user during SL-232's design round, on the responder's recommendation,
after the cheap alternative was falsified.

## What was found — `verified_sha` carries two incompatible value kinds

`stamp_verification` (`src/memory.rs:3524-3529`) writes `frame.checkout_state_id`
into `[git].verified_sha` under `--allow-dirty`, and `frame.commit` otherwise. One
field, two kinds, and **no discriminator on the record**. `[git].anchor_kind`
does not serve: it describes the *born* frame at `record` time, not the
verify-time stamp (389 memories: 328 `checkout_state` / 60 `commit`, against 59
attested).

Cross-tab at HEAD `377022dfa`, all 59 attested memories, ancestry guard run for
real (`probes/populations.py`, FAL-P2):

| `verified_sha` | guard | count | what it is |
|---|---|---|---|
| 40-hex, is a commit | ancestor | **25** | determinable — the only rows Checks 2/4 can speak about |
| 40-hex, is a commit | non-ancestor (exit 1) | **8** | ISS-257 proper: reachability undecidable |
| 40-hex, **not an object** | exit 128 | **2** | dangling — object absent from this clone (F-31's hazard) |
| **64-hex `checkout_state_id`** | exit 128 | **24** | never commit-anchored at all — an `--allow-dirty` stamp |

`commits_touching` collapses the last three rows to one `None`. So **34 of 59**
attested memories are silently unstaleable; reach is **42.4%**.

This also corrects `slice-232.md`, which cited *67 of 115 anchored*. Checks 2 and
4 both gate on `!verified_sha.is_empty()`, so the code-relevant denominator is
**attested (59)**, not anchored. The ratio survived re-measurement; the absolute
was overstated roughly twofold — the RV-313 F-1 failure mode, caught by
re-measuring rather than by inheriting.

## Why not the cheap discrimination — a falsifier, not a preference

The rejected option was a four-valued finding split by the stamp's **width**:
64 hex ⇒ not a commit. It was proposed as *lexical and therefore
checkout-stable*, which is why it looked cheaper than a schema change.

**Falsified.** `git init --object-format=sha256` is supported on git 2.54.0 and
yields **64-hex commit ids**. Probed, not reasoned about:

```
sha1:   commit id = 5be621b91e59433faca7e5af270f0600f1afcf50   (40)
sha256: commit id = ffc8b111cefaaf36231438f44726b6ae169e27fa6bea2bd5a23c3465aaba8410   (64)
```

The discrimination only runs on the `None` path — after the guard has already
failed — so on a sha256 repo the width rule labels **every** undeterminable row
"not commit-anchored", including every genuine non-ancestor commit. Not a rare
edge: total failure of the rule on that class of repo, and doctrine installs into
arbitrary client repos.

It does not even partition the sha1 case: the 2 dangling rows are 40-hex *and*
not objects, so catching them needs `cat-file -e` — the ref-set-dependent
instrument RV-307 **F-31 already refuted**. Two instruments, one non-total and
one refuted.

It would also introduce doctrine's **first** sha-width assumption. There are none
in `src/` today — no hardcoded 40/64, no `object-format` awareness anywhere.

So the correctness ordering is **(record it) > (flat) > (width)**. The width
option buys actionability by making a claim it cannot support; it is *less*
correct than saying less. Recorded at length because the idea is attractive and
cheap-looking, and the next reader will re-derive it.

## What the flat state costs, and what it does not

- **Conformance: nothing.** REV-041's obligation is that a surface emitting
  findings discharges the no-silent-over-trust prohibition *by emitting a
  finding, not by falling silent*. A flat state discharges it for all 34 rows.
  "Cannot determine drift from this stamp" is true whatever the cause — the flat
  state is under-informative, never false.
- **Actionability: bounded and draining.** 24 of the 34 need a re-verify, and
  the finding does not say so. But objective 1 is exactly what makes clean
  re-verification possible in a routinely-dirty corpus, so this is a one-time
  backlog the slice creates the remedy for — not a standing degradation of the
  findings surface. This narrows **R-G** rather than dismissing it.
- **No foreclosure.** A discriminator field is purely additive: the finding text
  changes and nothing else. The width heuristic would have had the opposite
  property — a rule that works on the maintainer's own repo is sticky.

## The convergence worth naming

This is the **third** time SL-232 has answered *"which instrument decides X?"*
with *"none — record it at the source"*:

| question | refuted instruments | answer |
|---|---|---|
| is this scope entry a path or a pattern? | character sniffing (`*`/`?`/`[`) | **the field it came from** (DEC-053) |
| is this entry expected to be unobservable? | existence (F-25), `rev-list --all` (F-31) | **declared on the record** (objective 3) |
| is this `verified_sha` a commit? | stamp width, `cat-file -e` | **record the kind** (IMP-325) |

Stated as a principle rather than three coincidences: **a property the writer
knows must not be re-derived by the reader from local repository state.** Every
derived instrument this slice tried reads state that a shallow clone, a pruned
repo, a dispatch worktree, or a different object format legitimately disagrees
about.

## Relations

- SL-232 (objective 7), ISS-257, IMP-325 (routed), IMP-318 / QUE-173 (same
  sequencing, OQ-A), DEC-053, DEC-054, DEC-020, REV-041, SPEC-007, RV-307 F-31,
  RV-313 F-1, risk R-G.
- Evidence: `.doctrine/slice/232/probes/populations.py` (HEAD `377022dfa`).
