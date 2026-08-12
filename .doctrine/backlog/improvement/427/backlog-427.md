# IMP-427: Split the capsule payload test band into live-bwrap and neutral

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred out of `SL-253` by `DEC-200`, which carves `conformance.rs`'s 186 tests
into **two** bands — kernel and payload — before the kernel/payload split.

The third band this defers: inside the payload band, roughly 42 tests need a live
`bwrap` and the rest do not. Separating them would let `just capsule-check` run
meaningfully on a host without `bwrap`, instead of the crate being reachable only
where the live rows can execute.

**The hard constraint on any implementation.** A third band must not become a
skip. `EX-14` and `DEC-156` are explicit that a claim standing behind a skip is
not a claim, and `ISS-342`'s fail-loud posture — the recipes fail rather than
skip on a host that cannot oblige — is what holds that line today. So the shape
to reach for is *selection* (a separately-invoked band, like `capsule-check`
versus `capsule-verify` already are) rather than *conditional execution*. The
justfile comment at the `capsule-check` recipe states the same distinction: "this
is a selection decision, not a skip, and the distinction is the whole point."

Related: `RV-352` `F-8` left open what a host that cannot satisfy the rows should
report at all. That question is upstream of this one and is not resolved by
`SL-253` either — an answer to it may change what this item should do.
