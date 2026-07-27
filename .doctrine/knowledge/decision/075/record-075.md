# DEC-075: Managed runs use the doctrine design command family

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 exposes its public run operations beneath a first-class
`doctrine design …` command family. The family owns starting a run, projecting
its current turn envelope, applying sparse declarations, inspecting status,
resuming, and materialising the authored design.

Commands identify the owning slice explicitly, and runtime state remains
slice-scoped. The shorter namespace is intended for repeated agent interaction;
it does not imply that design is independent of the governed slice lifecycle.

The pure state transition, declaration, projection, and persistence seams
should remain extractable if a contrasting future workflow demonstrates real
generality. V1 does not reserve `doctrine workflow …` or introduce a generic
workflow identity/configuration surface.
