# DEC-129: Egress allowlist and QUE-204 land in a follow-on slice

This is the largest piece of forward work SL-241 identified, and it exists as a
record rather than a slice. That gap is the point of writing it down.

## The one lever, seen from two ends

**From inside:** a capsule needs network egress for its build, and binary
on/off is the wrong shape. D-P05-14 settled that egress becomes an
**allowlist** — content varying per capsule kind, with agent hosts absent
entirely when no agent runs in that capsule. It deliberately did not say where
the work happens.

**From outside:** QUE-204 asks how a capsule obtains build inputs that git
cannot carry. Today `heavy` builds its web assets on site, every stage-3 cell
reaching `registry.npmjs.org`.

These are not two problems. An allowlist is exactly the mechanism by which a
capsule obtains build inputs, and QUE-204 is exactly the requirements document
for what the allowlist must contain. Settling them apart would make each
re-derive the other.

## What SL-241 actually produced here

A **feasibility result and nothing more.** `tinyproxy` and `iproute2` were
installed during the feasibility work; `socat` and `python3` were already
present; all four are DQ-4-clean. No allowlist was built. No remaining task in
the spike depended on one.

Resisting the pull to build it was the decision. A spike that implements the
model it is evaluating has stopped evaluating it.

## What the follow-on inherits — and must not redo

- **D-P05-14's reasoning** on why allowlist-not-binary, and why per-capsule-kind.
- **The F-P05-32 finding trail** from the feasibility work.

Do not re-derive either. They were expensive, they are archived in
`phase-sheets/phase-05.md`, and a fresh derivation will land somewhere subtly
different for no benefit.

## The standing cost until it lands

An `registry.npmjs.org` outage surfaces as `verify/suite-failed` — **the same
token a genuine test failure produces.** There is nothing in the refusal
vocabulary that distinguishes "your code is wrong" from "the network was down."
Any client whose build needs inputs the capsule cannot fetch is outside the
shape SL-241 measured, and `go-no-go.md` § 1 says so explicitly.

See [[DEC-131]] for the sibling posture decision about what the capsule's
environment may and may not reach.
