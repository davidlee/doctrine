# IMP-397: Capsule egress allowlist and build-input provisioning

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Lodged at SL-241 close so this stops existing only as a knowledge record.
**DEC-129** decided it lands in a follow-on slice; a decision is not a work item,
so until now the largest single piece of forward work the capsule spike
identified appeared in no `backlog list` and no `doctrine next`. This item is
that placement, and nothing more — the reasoning is DEC-129's and is not
restated here.

## The lever

Egress and build-input provisioning are **one lever seen from two ends**, which
is why DEC-129 refused to split them:

- **D-P05-14** settled *what* capsule egress becomes — allowlisted rather than
  binary, content per capsule kind, agent hosts absent when no agent runs — and
  explicitly did not place the work.
- **QUE-204** (`open`) asks how a capsule obtains build inputs git cannot carry.
  Today the `heavy` fixture's web assets are built on site per cell, so every
  stage-3 cell reaches `registry.npmjs.org`.

Splitting them would make each re-derive the other's reasoning.

## What is already known — do not re-derive

- **Feasibility is answered, and that is the whole of the result.** `tinyproxy`
  and `iproute2` were installed during SL-241's feasibility work; `socat` and
  `python3` were already present; all four are DQ-4-clean. **Nothing was built.**
- The follow-on inherits D-P05-14's reasoning and the F-P05-32 finding trail.
  Both are in the archived sheet at
  `.doctrine/rfc/025/evidence/phase-sheets/phase-05.md`.
- `go-no-go.md` § 1 ("What client shape this build shape means") and § 5
  (outstanding work) carry the scope this sits inside.

## Why it matters, stated as the live consequence

Until this lands, **an npm-registry outage lands as `verify/suite-failed` with
nothing to distinguish it from a real verdict.** A client project whose build
needs inputs the capsule cannot fetch is outside the shape the spike measured —
so this is not polish, it is the boundary of what the go/no-go actually claims.

## Shape

Almost certainly a slice rather than a chore — it is implementation of a model
that is not yet accepted, which is why SL-241 refused to carry it. Sequencing
against **CHR-054** (the REV that retires the census DELETE rows) is a real
question and is deliberately left open here: it may want the REV's governance
settled first, or it may be independent. Decide that when scoping, not now.

## Related

- **DEC-129** — the placement decision this item discharges.
- **QUE-204** — the open question it settles.
- **SL-241** — the spike that identified it (originates as D-P05-17, PHASE-05).
- **CHR-053** — the RFC-025 cleanup pass, which points at this item and does
  *not* cover it.
- **CHR-054** — REV scoping for the census DELETE rows; sequence unresolved.
