# DEC-068: Delegates return proposals and never mutate the run

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 v1 delegation is proposal-only. The coordinating run emits an attributed
assignment envelope containing assignment/run identity, base revision, bounded
inquiry nodes and question, relevant durable references, permitted context, and
the expected proposal shape.

A delegated session or agent returns a proposal envelope with reasoning,
evidence, attribution, and suggested sparse declarations. It has no authority
to write or advance the run. The coordinator is the sole writer and validates,
accepts, adapts, or rejects proposals through the ordinary mutation contract.

A proposal based on a stale revision is surfaced for reconsideration and is
never silently rebased or applied. V1 defines envelopes and local CLI exchange,
not harness-specific spawning, leases, write brokering, or a general transport
protocol.
