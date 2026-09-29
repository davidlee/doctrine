# ISS-502: Capsule conformance suite red on developer hosts and ungated

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

RV-411 F-13 (`doctrine show RV-411`). `crates/doctrine-control/src/conformance.rs`:
5 of 186 tests fail on the owner's NixOS host (bwrap 0.12.0). The row-2 and
row-9 payloads treat the top of every ambient `$PATH` entry as a bound root.

Also:
- three early-return "vacuous here" passes that libtest capture hides
- the admission test runs row 10 in-process, although the file documents that
  as racy
- row 13 uses uid/gid 1000, which is indistinguishable from the default first
  user on common distros
- about 2.5 minutes of wall time from 30 s timeouts

Nothing runs `just capsule-check` (no default recipe, no CI). Cargo.toml says
the capsule programme is stopped and the crate is pending triage. Resolve this
as part of that triage: fix and wire it into a gate, or retire the suite with
the crate.
