# ISS-278: ADR-008 D-B3 stale: claims claude workers cannot be OS-confined

ADR-008 decision D-B3 records that claude-arm dispatch workers cannot be
OS-confined, so the claude arm relies on cooperative (hook-enforced) write
confinement only. That is no longer true: the shipped dispatch skill confines
claude workers, and the `worker_commit` MCP gate plus the confined-spawn path
(`scripts/pi-spawn-confined.sh`) give the arm an OS-level boundary.

The claim is load-bearing when an agent reasons about arm parity — it reads as
"the claude arm is cooperative-only", which was superseded by SL-182 /
SL-185-era work and the current confined spawn.

## Provenance

Surfaced while authoring REV-043 during SL-237's design round. REV-043 names
the staleness as a follow-up but **deliberately does not amend ADR-008** — the
revision's three rows (ADR-006, SPEC-012, PRD-015) are scoped to SL-237's
primary-root change, and folding an unrelated correction into them would
smuggle governance drift through an unrelated approval.

## What resolution looks like

Confirm the current confinement posture against the shipped skill and spawn
script, then correct D-B3 through a Revision (ADR text is governance — it does
not get hand-edited). Check whether RSK-014 ("Claude subagent confinement is
unsolved", closed) and ADR-011's capability-altitude language carry the same
stale premise; if so, sweep them in the same revision.
