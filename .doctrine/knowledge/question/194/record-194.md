# QUE-194: Atomic knowledge checkpoint creation from design apply

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 aims to make timely DEC/QUE/ASM capture a reliable property of a managed
design interview. If `doctrine design apply` can only refer to a record created
through a separate `doctrine knowledge` ritual, the design may preserve the
very adherence failure it is intended to remove.

The complication is that design-run state is disposable runtime state while
knowledge records are authored, multi-file entities. A single filesystem ACID
transaction across both tiers is neither available nor desirable.

## Options

1. **Recoverable checkpoint declaration (recommended).** Let
   `doctrine design apply` accept a checkpoint declaration that creates one
   DEC, QUE, or ASM through the existing knowledge engine and then records its
   canonical reference while disposing the inquiry node. Treat the submission
   as atomic from the caller's perspective by journalling a narrow intent keyed
   by submission id. Validate before writing; never roll back an authored
   record; resume a known partial operation on retry. If a crash lands in the
   irreducibly ambiguous window after authored creation but before its reference
   is durably captured, block for explicit reconciliation rather than risk a
   duplicate. This is a design-specific recovery protocol, not a generic
   transaction framework.
2. **Separate knowledge command.** Require the agent to run
   `doctrine knowledge new`, link and settle it, then submit the returned
   canonical reference to `design apply`. This reuses existing commands with
   minimal new machinery, but retains a multi-command adherence burden.
3. **Prepared checkpoint only.** Have `design apply` validate and stage a
   checkpoint proposal or print the exact follow-up knowledge command, without
   authoring it. This makes intent visible but still depends on the agent
   completing a separate ritual.

The recommendation is option 1 because incremental durable checkpoint creation
is a primary product outcome and evaluation signal for SL-233. The recovery
protocol must remain narrow and conservative: authored truth is never deleted,
silent duplicate creation is forbidden, and ambiguous partial failure requires
human reconciliation.
