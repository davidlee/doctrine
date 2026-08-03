# CHR-054: Scope a REV for the mechanism-census DELETE rows

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Lodged by **SL-241 PHASE-06 (EX-10)**, conditional on the verdict being go — and
it is: `.doctrine/rfc/025/go-no-go.md` returns **go, scoped to Linux/bwrap for a
client of this build shape**, for the ingestion and confinement halves of the
capsule model.

## What needs doing

`.doctrine/rfc/025/mechanism-census.md` marks a set of incumbent mechanisms
**DELETE** under the capsule model — the worker marker, the role-detection rules,
the security-significant hooks, and the rest of the machinery whose job the
sandbox takes over. Those verdicts touch **governance**: shipped assets,
requirements, and at least one ADR describing the incumbent posture. Governance
change routes through a **Revision** (ADR-013), not through direct edits.

This item is the **scoping** step: work out what the REV's boundary is before
anyone writes it.

## Read the scope off the verdict, not off the census

The go/no-go's scope is load-bearing here and this is the most likely place for
it to get lost. Specifically:

- **The verdict is Linux/bwrap only.** Every environment-conditional census row
  — anything whose replacement boundary is a mount namespace, `--unshare-all` or
  `ulimit` — is **outstanding for macOS**. A REV that deleted an incumbent
  mechanism on all platforms would be claiming more than the spike measured.
- **Conflict/staleness *resolution* is out of evidence** (QUE-202). The capsule
  model's *refusal* of a second result from one base is proven safe, total and
  content-blind; **admission is not designed**. Any census row whose replacement
  depends on admission is not ready to delete.
- **QUE-200 is still open.** The ingestion mechanism — fetch-into-quarantine vs
  bundle — did not settle on the spike evidence (SL-241 PHASE-06 T8, D-P06-9).
  Rows whose verdict differs by mechanism cannot be closed until it does.
- **The count is not "16/16".** Sixteen hazard rows with a capsule-model boundary
  or a recorded dissolution, plus two regression legs that count toward nothing.

## Sequencing

Almost certainly wants **CHR-053** (the RFC-025 cleanup pass) done first — the
census's own B1 note is one of that item's four pieces, and scoping a REV against
a document known to be stale is wasted work.

## Related

- **SL-241** — the capsule spike; `.doctrine/rfc/025/go-no-go.md` § 1 is the
  scope this REV must inherit and § 5 the outstanding work it must not absorb.
- **RFC-025**, `.doctrine/rfc/025/mechanism-census.md` — the subject.
- **CHR-053** — the cleanup pass; do that first.
- **ADR-013** — governance dependency routes through a Revision.
- **QUE-200**, **QUE-202**, **QUE-204** — all open; each bounds what this REV can
  claim.
