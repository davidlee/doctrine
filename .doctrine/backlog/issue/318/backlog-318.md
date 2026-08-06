# ISS-318: Declaration keys inert at the subject's kind are silently ignored

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A `design apply` payload can carry keys that mean nothing and be accepted
anyway: the submission bumps the revision, writes a receipt, prints no change
row, and exits 0. The class is the item, and it has **two axes**.

- **The declaration axis.** `Declaration` is one flat wire struct whose fields
  are each honoured at one subject kind and inert at the rest, and nothing checks
  that correspondence. Observations 1 and 2, in opposite directions.
- **The envelope axis.** `ApplyRequest` cannot carry
  `#[serde(deny_unknown_fields)]` at all, so a key it does not know is discarded
  before any of the run's own checks see it. Observation 3.

The title names the first axis only, because the second was added later; the
item covers both.

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

## Observation 3 — an unknown key anywhere at the top level

The envelope axis. Witnessed on SL-248's design run, 2026-08-06, while drafting
`sec-7`: the payload shape for declaring a section body is not discoverable from
`design apply --help` (which documents only `--input`), and `design show`'s
trailing hint line demonstrates the inquiry form alone. Probing for the schema
cost three revisions and established nothing, because every probe was accepted:

```json
{"…envelope…", "sections": []}                  → revision 47, no change rows
{"…envelope…", "sections": [{"foo": "bar"}]}    → revision 48, no change rows
{"…envelope…", "zzz_nonsense": [{"a": 1}]}      → revision 49, no change rows
```

The answer — sections are declared through `declare` with a `body` field — was
findable only by reading `src/design_run/submission.rs`.

### Cause

`ApplyRequest` carries `#[serde(flatten)] envelope: SubmissionEnvelope`, and
serde cannot reconcile `flatten` with `deny_unknown_fields` — the flattened
field's own keys would be refused as unknown. The type says so in its own doc
comment (`submission.rs:117`). Every *inner* submission type does carry
`deny_unknown_fields`; the outermost one, which is the only one a caller
hand-authors from scratch, structurally cannot.

### The client-facing form is worse

Two things make this sharper for an installed client than for this repo.

1. **No schema handshake.** `SubmissionEnvelope` is `run_uid` /
   `known_revision` / `submission_id`. `known_revision` is optimistic
   concurrency, not version negotiation, so nothing on the wire can notice that
   a caller and a binary disagree about the payload shape.
2. **Skew is the normal state.** Doctrine ships skills that instruct an agent to
   send a given payload shape. A client's `doctrine` binary and its installed
   skills drift apart independently; this repo's do not, because we build from
   the tree. The stale-`~/.cargo/bin/doctrine` sibling recorded under Related
   is this same defect with skew as the trigger rather than an agent guessing —
   and that binary reports the same `--version` as the tree build.

### Severity, honestly

Mostly fail-safe rather than corrupting. The run is gated, so a dropped
mutation leaves its condition undischarged and the stage refuses to advance —
a dropped section declaration surfaces at `drafting → reviewing`. The cost is
wasted turns plus a false progress report to the operator.

The exceptions are the run-level acts nothing downstream gates: `traversal`
(harmless — attention simply did not move) and **`review_policy`**, where a
silent drop leaves the agent believing it changed which reviewer lanes the run
requires while no later condition contradicts it.

### Blast radius elsewhere

Checked across all 17 `serde(flatten)` sites, 2026-08-06. Most cannot exhibit
this: `status.rs:268`, `rec.rs:571` and `review.rs:1895` are `Serialize`-only
output shapes, and `value.rs`, `estimate.rs` and `memory.rs` flatten a
`BTreeMap` deliberately to *preserve* unknown keys for round-tripping — the
opposite failure mode. The deserialising input surfaces are `ApplyRequest`,
`ConductConfig` (`[conduct]` in a client's `doctrine.toml`),
`ConceptMapMutation` (map-server HTTP), and `observation::wire::Envelope` —
the last guarded upstream, since `observation/request.rs` does carry
`deny_unknown_fields` on the input path. `ApplyRequest` is the only one a
caller hand-authors against a schema it cannot introspect.

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

Options 1 and 2 address the **declaration axis** only. Neither touches
observation 3, whose keys are discarded by serde before the run's own checks
run at all.

3. **Check the top-level key set before typed deserialisation.** For the
   envelope axis: parse `--input` to a `serde_json::Value`, compare its
   top-level keys against a closed set, refuse naming the unrecognised key, then
   deserialise typed as now. This is already the shape the type reaches for —
   `ApplyRequest::WRITER_ACTS` is a closed, named-constant vocabulary of the
   run-level keys, and its doc comment states the governing intent outright:
   *"a new field that can change state and did not join this list is the gap,
   and an exhaustive list is at least visible in review."* The full key set is
   that list plus the non-writer fields plus `SubmissionEnvelope`'s three. Cheap,
   preserves the wire, and needs no schema version.

   Weaker variant if refusal is judged too strict: report unrecognised keys on
   the `apply` output rather than refusing. That closes the *silence*, which is
   the actual defect, without changing what is accepted.

Option 1 is proportionate on its own; option 2 is the same argument the design
run has already applied elsewhere, so it is worth weighing rather than
dismissing. Option 3 is close to independent of both and could land first. Either
way the fix should be total over the field set, not one patch per instance
recorded here.

## Related

- **IMP-403** (knowledge facets are systematically unfilled) — its lead 2 is the
  other half of observation 2. IMP-403 notes that `CreateRecord` offers no facet
  slot, so a checkpoint cannot carry the ruling the agent has in hand; this item
  records what happens when the agent tries anyway through the nearest-looking
  key. One is a missing slot, the other is a silent sink, and together they are
  why records mint hollow. A fix to either should be weighed against the other:
  refusing `body` here without giving `CreateRecord` somewhere to put prose just
  converts a silent loss into a dead end.
- Sibling finding from SL-247's session, separately observed: a stale
  `~/.cargo/bin/doctrine` silently drops unknown `apply` fields (it reports the
  same `--version` as the tree build), so an older binary narrows the payload
  and reports success. Originally recorded here as "different mechanism, same
  class" — observation 3 establishes it is the *same* mechanism with a different
  trigger, and folds it into the envelope axis.
- `mem.pattern.serde.flatten-forbids-deny-unknown-fields`
  (`mem_019fd03e13397240b4eb05af218f5cf5`) — the same root cause seen from the
  retirement direction: a *removed* field also becomes a silent no-op, which is
  how three e2e fixtures went on sending a deleted `evidence` key for two tasks
  with every suite green.
- SL-247: the slice whose design run surfaced observation 1 and the stale-binary
  sibling. SL-248's run surfaced observations 2 and 3.
