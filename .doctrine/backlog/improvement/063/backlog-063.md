# IMP-063: Supersession for POL/STD/slice — grow superseded vocab so supersede_policy returns Some

SL-062 close-time follow-up F2 (design §9, F-C). SL-062's `supersede` verb is
ADR-first because only ADR has a `superseded` status; POL/STD/slice return `None`
from `supersede_policy`. To flip them on (zero verb-shell change):
- add `superseded` to the kind's status const + enum (and `lifecycle::classify` for
  slice's ordered FSM);
- ensure a `superseded_by` carve-out field (POL/STD already seed it; slice needs it
  added + flip the slice `supersedes` relation rule to `LifecycleOnly`);
- then `supersede_policy(KIND)` returns `Some`.
