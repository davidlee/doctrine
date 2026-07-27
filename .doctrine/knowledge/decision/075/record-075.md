# DEC-075: Managed runs use the doctrine design command family

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 exposes its public run operations beneath a first-class
`doctrine design …` command family. The family owns starting a run, projecting
its current turn envelope, applying sparse declarations, resuming, and
materialising the authored design.

Commands identify the owning slice explicitly, and runtime state remains
slice-scoped. The shorter namespace is intended for repeated agent interaction;
it does not imply that design is independent of the governed slice lifecycle.

The pre-existing `doctrine slice design <ID>` command is retained as a
deprecated compatibility shim. When a managed run exists it delegates to
`doctrine design materialise <ID>` through the same implementation and
foreign-edit guard. When no run exists it preserves the incumbent scaffold-only
behaviour: it creates the design template only when `design.md` is absent,
emits a deprecation warning directing new work to `doctrine design start`, and
otherwise preserves the existing no-clobber refusal with
`doctrine design start --from-design` as the migration path. The shim never
creates or reconstructs runtime state.

The two paths are mutually exclusive rather than independent live-run writers.
New documentation, help examples, generated boot guidance, and agent guidance
use the top-level family.

Read naming follows the established CLI convention: `show` is the bounded
default projection, and `show --full` explicitly requests protocol and inquiry
map detail that may scale with the run. V1 does not add a design-specific
`inspect` verb with inverted semantics.

The pure state transition, declaration, projection, and persistence seams
should remain extractable if a contrasting future workflow demonstrates real
generality. V1 does not reserve `doctrine workflow …` or introduce a generic
workflow identity/configuration surface.
