# ISS-318: Declaration keys inert at the subject's kind are silently ignored

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`Declaration` is one flat wire struct whose fields are each honoured at one
subject kind and inert at the rest. Nothing checks that correspondence, so a key
sent to the wrong kind is accepted, bumps the revision, writes a receipt, and
does nothing. Two instances are recorded below, in opposite directions; the
class is the item.

## Observation 1 — `dispose` on an `inq-` subject

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

### Cause

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

## Observation 2 — `body` on a `cp-` subject

The mirror image, witnessed on SL-248's design run, 2026-08-06. Six checkpoint
dispositions with `form = "create"` each carried a prose `body` alongside the
record's `kind`/`title`. All six were admitted; DEC-155..DEC-160 were minted
with **empty** bodies and the prose was discarded without a word.

```json
{"subject": "cp-1", "disposes": "inq-1",
 "dispose": {"form": "create", "kind": "decision", "title": "…"},
 "body": "…prose that went nowhere…"}
```

### Cause

`Declaration::body` is *section* prose. Its only consumer is
`src/design_run/run.rs:1323`, guarded on `derived.section_digests.get(id)` —
so on a `cp-` subject the guard falls through and the field is accepted and
ignored.

The `create` contract itself is fine: a disposition carries
`kind`/`title`/`slug`/`acceptance`, and the record's prose is authored
separately into its `.md`. The defect is that nothing said so.

### Why it costs

Six records shipped hollow. Caught by the operator rather than by the tool,
then ~15k tokens of prose re-authored from scratchpad payloads.

## Why it costs (general)

The failure is indistinguishable from success at the CLI. An agent gets exit 0,
a fresh revision and a receipt, so the natural next step is to carry on. The
detection path is a `design show` round-trip plus reading totals carefully
enough to notice `resolved=0` — or, for observation 2, reading back a minted
record's `.md`.

The wire shape makes the mistake easy to reach: every one of these is a field
on `Declaration`, so each is spellable on every subject kind while only one kind
honours it. The relationships ("a `cp-` disposes an `inq-`"; "`body` is section
prose") live in doc comments, not in the type.

Note the codebase already applies rule/record correspondence to the sibling
field — `disposition` is required present exactly on `ReviewDisposed`, checked
in `src/design_run/admission.rs` so the failure is a typed refusal naming the
act. That machinery exists; it just does not cover the subject-kind axis.

## Candidate fixes

1. **Refuse.** A typed refusal when a declaration carries a key inert at its
   subject's kind — naming the subject, its kind, the key, and the kind that
   would honour it. Cheapest, and consistent with how the run refuses
   elsewhere. Wants a single table of key → honouring kind so the check is
   total rather than a growing list of `if`s.
2. **Unrepresentable.** Split `Declaration` into per-subject-kind wire types, so
   a disposition on an inquiry subject and a body on a checkpoint cannot be
   spelled. Larger change; matches the codebase's stated preference for
   unrepresentability over checks (see the `AgentAct` vs `ActKind` split, which
   makes an agent-authored `DesignAccepted` unrepresentable for this reason).

Option 1 is proportionate on its own; option 2 is the same argument the design
run has already applied elsewhere, so it is worth weighing rather than
dismissing. Either way the fix should be total over the field set, not two
patches for the two instances recorded here.

## Related

- **IMP-403** (knowledge facets are systematically unfilled) — its lead 2 is the
  other half of observation 2. IMP-403 notes that `CreateRecord` offers no facet
  slot, so a checkpoint cannot carry the ruling the agent has in hand; this item
  records what happens when the agent tries anyway through the nearest-looking
  key. One is a missing slot, the other is a silent sink, and together they are
  why records mint hollow. A fix to either should be weighed against the other:
  refusing `body` here without giving `CreateRecord` somewhere to put prose just
  converts a silent loss into a dead end.
- Sibling finding from the same session, separately observed: a stale
  `~/.cargo/bin/doctrine` silently drops unknown `apply` fields (it reports the
  same `--version` as the tree build). `ApplyRequest` carries no
  `deny_unknown_fields`, so an older binary narrows the payload and reports
  success. Different mechanism, same class — a submission that looks applied
  and was not.
- SL-247: the slice whose design run surfaced observation 1 and the stale-binary
  sibling. SL-248's run surfaced observation 2.
