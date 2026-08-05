# ISS-318: Dispositions on an inquiry subject are silently discarded

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observation

A `design apply` payload that declares a `dispose` against an **inquiry**
subject is accepted, bumps the run revision, writes a receipt — and changes
nothing. No refusal, no warning, no change row.

Witnessed on SL-247's run `dr-019fd215-bf7d-7612-8133-9e5acf77b503`,
2026-08-05. The batch below returned `revision 8 stage exploring` and nothing
else; `design show` then reported `changes: none since revision 7` with totals
unchanged at `open=8 resolved=0`.

```json
{"subject": "inq-1", "dispose": {"form": "create", "kind": "decision", ...}}
```

The same batch re-expressed against checkpoint subjects applied cleanly and
minted DEC-152:

```json
{"subject": "cp-1", "disposes": "inq-1", "dispose": {"form": "create", ...}}
```

## Cause

`plan_checkpoints` (`src/commands/design.rs:882`):

```rust
if declaration.subject().kind() != IdKind::Checkpoint {
    continue;
}
```

Any non-checkpoint subject is skipped. The `continue` is correct for a
declaration that carries no disposition — that is the common case, a plain node
edit — but it also swallows the case where a disposition **is** present and the
subject is simply the wrong kind. The two are indistinguishable at this point,
and only the second is a caller error.

Note the neighbouring comment already reasons about the other skip in this
function ("a checkpoint declaring no disposition is skipped rather than refused
here: the pure core owns that refusal") — so the deliberate-silence argument
has been made for the converse case, where the core does own a refusal. Here
nothing downstream owns it.

## Why it costs

The failure is indistinguishable from success at the CLI. An agent gets exit 0,
a fresh revision and a receipt, so the natural next step is to carry on. The
detection path is a `design show` round-trip plus reading totals carefully
enough to notice `resolved=0`. On SL-247 this burned several turns and produced
a wrong root-cause hypothesis.

The wire shape makes the mistake easy to reach: `dispose` is a field on
`Declaration`, so it is spellable on every subject kind, while only one kind
honours it. The relationship "a `cp-` disposes an `inq-`" lives in the
`disposes` field's doc comment, not in the type.

## Candidate fixes

1. **Refuse.** A typed refusal when a declaration carries `dispose` and its
   subject is not a checkpoint — naming the subject, its kind, and the `cp-`
   form that would work. Cheapest, and consistent with how the run refuses
   elsewhere.
2. **Unrepresentable.** Move `dispose`/`disposes` off `Declaration` into a
   checkpoint-specific wire type, so a disposition on an inquiry subject cannot
   be spelled. Larger change; matches the codebase's stated preference for
   unrepresentability over checks (see the `AgentAct` vs `ActKind` split, which
   makes an agent-authored `DesignAccepted` unrepresentable for this reason).

Option 1 is proportionate on its own; option 2 is the same argument the design
run has already applied elsewhere, so it is worth weighing rather than
dismissing.

## Related

- Sibling finding from the same session, separately observed: a stale
  `~/.cargo/bin/doctrine` silently drops unknown `apply` fields (it reports the
  same `--version` as the tree build). `ApplyRequest` carries no
  `deny_unknown_fields`, so an older binary narrows the payload and reports
  success. Different mechanism, same class — a submission that looks applied
  and was not.
- SL-247: the slice whose design run surfaced both.
