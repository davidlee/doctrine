# REV REV-065 — Six routes and the lock route check

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-270` changes the review-ledger route vocabulary and makes an unrouted severe
finding gate the design-run lock. `ADR-007` and `SPEC-032` both state the old
five-route set, and `SPEC-032` says an unknown route "gates nothing". Raised by
`RV-400` F-2; approved by the owner 2026-09-27, to be applied at `SL-270`'s
reconciliation.

Decisions: `DEC-330` (route split), `DEC-326` (lock check, as amended by
`RV-400` F-7).

### Amendment 1 — ADR-007 D-C5: the route set

| Where | Before | After |
|---|---|---|
| adr-007.md D-C5 | An optional `route` (`review \| demonstrate \| probe \| control \| owner-fix`), on the same terms, records where the answer sends the finding. | An optional `route` (`review \| demonstrate \| probe \| control \| dedupe \| refresh`), on the same terms, records where the answer sends the finding. A legacy `owner-fix` stays readable. The design-run lock refuses while a disposed severe finding on its review pass carries no known route (DEC-326); the ledger's own status and `conclude` are unaffected. |

### Amendment 2 — SPEC-032: vocabulary and the gating exception

| Where | Before | After |
|---|---|---|
| vocabulary table, `Route` row | `review` `demonstrate` `probe` `control` `owner-fix` | `review` `demonstrate` `probe` `control` `dedupe` `refresh` |
| "closed on write, open on read" paragraph | It is not a defect and gates nothing. | It is not a defect and gates nothing in the ledger. One reader outside the ledger gates on route: the design-run lock refuses while a severe (`blocker`/`major`, or unknown severity) finding whose status is not open or withdrawn (an out-of-vocabulary status counts), and that is not already an undisposed blocker, carries no known route — absent, a legacy `route:` disposition prefix, or an out-of-vocabulary value, each named (DEC-326). |
| "Unknown disposition, route, act or role" bullet | … is rendered verbatim, gates nothing … | … is rendered verbatim and gates nothing in the ledger (route: see the design-run lock exception above) … |
