# QUE-176: Which harnesses expose trustworthy usage instrumentation?

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Question

Which supported harnesses and model providers expose trustworthy,
machine-readable token or resource-usage measurements, at what lifecycle point,
and with which counter definitions and scopes?

## Why it remains open

Instrumentation availability and semantics vary by harness and provider. The
observation substrate can define typed storage and late correlation without
pretending that every execution path can populate them.

This question does not gate SL-231's core ledger or capture interface. It should
guide per-harness interoperability work and determine which adapters can
truthfully emit usage observations.

## Resolution evidence

Resolve per integration only after inspecting its actual accounting interface
and documenting:

- available counters and units;
- whether values are provider-reported, harness-calculated, or unavailable;
- the measured interval or execution scope; and
- when the completed measurement becomes observable.
