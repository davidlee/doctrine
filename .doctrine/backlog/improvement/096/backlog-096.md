# IMP-096: Requirements capture and refinement skills for the reconcile loop

The skills touched by SL-098 make implied-requirement discovery and orphan
placement a natural part of the design→plan→audit→reconcile→close loop. But
as requirements become more central to the loop, dedicated skills will likely
emerge:

- **capture / formalise** — refining an informal obligation into a well-formed
  requirement statement, assigning kind, proposing home altitude
- **refine / renegotiate** — massaging a requirement to fit alongside its
  neighbours in a spec, splitting or merging requirements
- **placement** — the altitude-and-home decision that SL-098 currently places
  inside `/reconcile`, which could grow into its own skill

When any of these skills materialize, they must integrate at the touchpoints
SL-098 defines: the `REQ-DNN` handoff in `design.md`, the `[requirements]`
manifest in `plan.toml`, the orphan section of the reconciliation brief, and
the REV `introduce` path in reconcile.
