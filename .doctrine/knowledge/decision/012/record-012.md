# DEC-012: Observation identity is UUID-native

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observations have UUID-native identity. Recording an observation does not allocate
a numbered doctrine entity id, and its durable identity does not depend on later
interpretation or significance.

SL-231 may make an observation directly addressable by UUID, but it does not add a
promotion mechanism.

## Rationale

Observations are expected to be numerous, independently captured, and merge-safe.
Numbered ids would impose authored-entity significance and allocation coordination
on every occurrence. UUID identity preserves cheap, collision-free capture without
prejudging which observations matter.

## Deferred extension

A later capability may give a significant observation an ergonomic numbered
citation — for example when it supports an EVD record cited by a design. Such a
capability must preserve the observation's UUID as its native identity and add an
explicit alias, wrapper, or promotion record; it must not silently replace or
renumber the source observation.

The promotion model, numbered kind, and citation semantics are outside SL-231.
