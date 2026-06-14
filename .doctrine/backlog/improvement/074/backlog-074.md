# IMP-074: Transactional creation-apply for revision: auto-land introduce/create rows once spec::add_requirement/create_spec engine helpers exist (SL-066 OQ-3 / external B2)

Source: SL-066 design §5 / §4.5 / OQ-3 (carried). v1 `revision apply` auto-lands
`status` rows ONLY — they ride the engine-callable `requirement::set_status`. The
creation ops (`introduce`/`create`) are surfaced-for-manual because `spec req add` /
`spec new` are non-transactional CLI handlers (`spec.rs:826` "NOT transactional by
design"); auto-applying them would risk orphaned half-writes the one apply commit
cannot undo (external B1+B2).

When transactional engine helpers `spec::add_requirement` / `spec::create_spec`
exist, `revision apply` can auto-land `introduce`/`create` rows too: allocate the
id, member it under the frozen `new_label` + `member_of` SPEC, and **back-fill the
`ChangeRow.allocated` field** (today the operator hand-fills it, design.md:228 — this
is the field's automated producer, RV-029 F1). Additive — no model change. The
all-or-nothing apply atomicity would then extend across heterogeneous row kinds (the
OQ-2 heterogeneous-atomicity that v1 dropped).

Blocked on the spec engine-helper extraction (the handler→engine refactor v1
deliberately avoided to dodge an ADR-001 layering violation).
