# ISS-368: backlog needs accepts targets doctrine needs refuses

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`backlog::run_needs` validates each prerequisite with `kinds::ensure_ref_resolves`
alone (`backlog.rs:1963-1964`) and never applies an admissible-target predicate —
`is_admissible_dep_target` appears nowhere in `backlog.rs`. So the backlog-scoped
verb accepts **any resolvable kind**.

The kind-neutral sibling refuses three classes at `commands/dep_seq.rs:92-97`:
`RV`, knowledge records, and governance docs. Governance is excluded on purpose —
depending on governance routes through a Revision, never the evergreen doc
(the `SL-060` invariant, `ADR-013`).

```
doctrine needs         ISS-1 ADR-1   → refused
doctrine backlog needs ISS-1 ADR-1   → accepted, edge written
```

So one verb writes edges its sibling would not, including edges onto governance
docs that `ADR-013`'s routing rule exists to prevent. `ISS-046` was the mirror
defect on the same surface (targets wrongly *refused*) and is closed; this is the
over-permissive direction and was never raised.

## Why it is filed separately from SL-238

Found as `RV-358` `F-5` against `SL-238`'s design, and `SL-238` §6 fixes it by
injecting the shared gate as `DepSeqOps.admit_target` — one gate, one message, no
new module edge. Filed here anyway on two grounds: it is a live defect in shipped
behaviour whether or not `SL-238` lands, and the fix is a **deliberate refusal of
input accepted today**, which is a behaviour change that deserves its own record
rather than riding a design section.

It also matters to `SL-238`'s reasoning: §2 and §3 argue the probe never meets an
`Unavailable`-class target, and that claim is only true *going forward* once this
gate exists. Today such a target is authorable through this path.
