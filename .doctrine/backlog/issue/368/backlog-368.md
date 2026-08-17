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

## Closed 2026-08-17 by SL-238 PHASE-08 (`df6185164`)

`run_needs`' prerequisite loop now resolves with `kinds::parse_resolvable_ref` and
then applies the shared gate through the injected pointer:

```rust
let (tkref, _tid) = crate::kinds::parse_resolvable_ref(&root, prereq)
    .with_context(|| format!("prerequisite `{prereq}` does not resolve"))?;
(ops.admit_target)(tkref.kind, prereq)?;
```

`admit_target` is `commands::dep_seq::ensure_admissible_dep_target`, extracted from
the `ensure!` `resolve_dep_seq_src` already ran (`1b3b90775`), so the two verbs
refuse with the SAME BYTES rather than with two vocabularies that agree today. The
gate's `?` sits outside the resolve's context deliberately — an anyhow frame over it
would prepend `prerequisite … does not resolve` and make the two messages differ.

Pinned by `backlog_needs_refuses_an_inadmissible_target_kind`
(`tests/e2e_dep_seq_verbs.rs`), which asserts the two verbs' stderr are EQUAL over
`RV`, `REC` and `ADR` fixtures — live run against live run, not against a
transcribed literal — with `QUE-001` as the positive control that the gate still
admits what it should. `ensure_admissible_dep_target_refuses_in_one_voice_with_resolve`
(`src/commands/dep_seq.rs`) is the in-module half.

One correction to the report above: the kind-neutral verb does **not** refuse
knowledge records. `SL-158` `D2` admits `ASM/DEC/QUE/CON/EVD/HYP` as dep/seq
targets; the refused classes are `RV`, `REC` and governance docs. The defect and its
fix are unaffected — `backlog needs` was accepting all three of those.
