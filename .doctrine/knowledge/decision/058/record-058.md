# DEC-058: Design handover short-circuits to the managed run

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 v1 includes a design-run-aware `/handover` branch. When an active managed
design run exists, handover first ensures its runtime checkpoint and durable
semantic records are current, then emits the stable run reference/resume command
instead of constructing a parallel continuation prompt or `handover.md`.

Residual prose is emitted only for explicitly identified context that the run
projection and durable records cannot represent. The integration is specific to
managed design runs; v1 does not introduce a universal resumable-state provider
protocol or move other skills onto the run mechanism.
