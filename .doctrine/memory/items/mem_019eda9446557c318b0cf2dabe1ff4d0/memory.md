# IMP-096 requirements-capture skills + IMP-097 altitude framework

Two deferred follow-up IMPs from SL-098:

**IMP-096: Requirements capture and refinement skills.**
Integration touchpoints:
- `design-requirements.toml` file (format defined by SL-098)
- `plan.md` `## Requirements verification` prose list
- Audit brief `#### Orphaned requirements (REV introduce)` sub-section
- Reconcile REV `introduce` path (step 4f)

**IMP-097: Altitude assessment framework.**
Blocking dependency for reliable orphan placement. Until resolved:
- Reconcile skill carries a `/consult` guardrail
- Placement relies on existing spec `c4_level`/`descends_from` as reference points
- Ambiguous altitude questions trigger `/consult` rather than guessing
- Stuck orphans are non-terminal for close
