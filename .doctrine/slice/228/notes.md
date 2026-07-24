# SL-228 — notes

## Harvest

fresh-as-of: design drafted + self-reviewed, pre-inquisition · head `0603f11f`+design commit

### Produced
- `extraction.md` (committed `0603f11f`) — as-built funnel state machine, crux
  verdicts, per-verb invariant table. Design input; stays useful post-close as
  the D7 artifact's ancestor.
- `design.md` — full design, D1–D8 locked with User, internal adversarial pass
  integrated (6 findings, §-noted). **Pending external inquisition (codex).**
- `design-target` selectors recorded (15).
- Scope `## Follow-Ups` reconciled to design (OQ-2/NEW-OQ-A/NEW-OQ-B → D1/D6/D7).

### Learned (durable sinks already hold these)
- The reverse-diff resync trap (`reset --keep` broken when ref advanced under
  checkout) is memory-pinned; design §5/R3 rides the `restore`-based idiom.
- Two-altitude finding (sub-funnel has no `select_guidance` node) — in
  extraction §3; drove D4's "new oracle, not carve-out" framing.

### Open
- **External inquisition** of design.md with codex — parked for a fresh agent
  (see `handover.md`). Design NOT user-approved-final until that closes.
- Plan-phase items deliberately deferred: OQ-5 benchmark harness shape; hook
  script asset path (selector added when picked); `NextCore.command` binary-name
  rendering; REQ-287 prose mapping (ship-time REV per Non-Goals).
