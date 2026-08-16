# Review RV-360 — design of SL-256

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

External adversarial pass over all four sections of SL-256's design. The review
probes whether the proposed `ActRecorded` event makes every successful act write
observable without weakening admission-before-mutation, whether the shared
admit-store-emit seam is genuinely non-bypassable, and whether the
`READABLE`/`EMITTABLE` split models historical compatibility and live writer
coverage completely rather than by convention.

The factual pass re-derives every direct `ChangeEvent::ALL` consumer, every
`admit_against` caller, the ownership and reader roles of `RecordedAct`, the
ordering of `Pending` rows through `Applied::rows`, and the existing
`every_event_fixture` inputs. The suite-fence argument is checked across every
`tests/e2e_design*.rs` file: change-log reads, positional indexing, exact counts,
and stdout comparisons are searched independently, with known-positive patterns
used to prove the searches can detect the prohibited form.

The invariants held against the subject are REQ-478's three acceptance criteria;
SL-256's R1/R2/R4/R5 compatibility, bound, coordination, and payload-width risks;
ADR-001's leaf ← engine ← command layering; STD-001's single-source named
constants; STD-002's durable reference forms; STD-003's read-path scope; and
POL-002's prohibition on host-convention or transient-local-state coupling. The
likely bodies are `change_log.rs`'s manual roster and serde/token mapping,
`run.rs`'s public construction and storage seams, `snapshot.rs`'s group writers
and legacy parser pins, the e2e fixture ladder and change-log assertions, and the
coverage recipe promised at closure.
