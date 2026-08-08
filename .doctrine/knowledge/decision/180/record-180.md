# DEC-180: Capsule suite may red the tree on hosts without bubblewrap

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

`SL-248` ships an admission test that asserts `Admitted` **unconditionally**, and
that is accepted. On a host that cannot run the bubblewrap backend, this
project's `cargo test` fails — and because `SL-248` also puts
`crates/doctrine-control` in `default-members`, the failure arrives in
`just check` rather than only in a release gate.

No phase owes a mitigation. No conditional skip is to be added.

## Why

Conditioning the assertion on backend availability reintroduces exactly the
green skip `DEC-156` forbids: a suite that reports success because it never ran.
`design.md` `sec-9` residual 3 states the cost and declines to hide it.

The slice owner accepted it at plan time on two grounds:

1. In practice nobody outside this project runs its tests, so the population
   actually exposed is one.
2. The gate that matters is the **end of the slice**, not the end of each phase
   — phases are built inline on `edge`, and a transiently red inner loop between
   phase landings is tolerable where an unsound admission verdict is not.

## What this does and does not settle

**Settled:** the local-host behaviour. A reader meeting residual 3 should not
treat the local case as an open question or re-open it.

**Still open:** the CI ruling. `design.md` `sec-9` residual 3 is explicit that
the affected set is not "macOS" but *any host, or any nesting, that denies what
the backend needs* — a seccomp-filtered CI runner as much as an agent sandbox,
because row 5's probe arm needs the socket bubblewrap opens to bring up loopback
in a fresh network namespace. The design says the choice — require a runner that
permits the namespaces, or accept that admission is established on developer
machines and release hosts only — is owed by whichever slice first runs this
suite in CI. This decision does not pre-empt that one.

`backend verify` remains the descriptive path on such a host, reporting
`NotAdmitted::Unavailable` naming what is missing and what would satisfy it
(`POL-002` facet 3), rather than failing opaquely.

## Where it lands

`SL-248` `plan.md` § *What each phase changes about the tree* records this
against PHASE-10, which is the phase whose landing makes the assertion complete.
