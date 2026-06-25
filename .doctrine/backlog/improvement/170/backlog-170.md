# IMP-170: UX review of relation-authoring CLI surfaces (coverage + consistency)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

Spawned by SL-153 (CLI verbs for the last hand-edit-only spec-internal edges).
SL-153 closes `descends_from`/`parent`/`interactions` but surfaced a wider gap:
the relation-authoring CLI surface is not uniformly modelled.

Concrete known instance: product `parent` (PRD→PRD) is authorable (SL-065 added
`Spec.parent` + render + `build_registry` `on_product` validation) and SL-153 lets
`spec edit --parent` set it, but `RELATION_RULES` declares **no** PRD-parent row and
the product template carries no `parent` example. The table under-declares reality.

## Scope

UX/consistency review across **all** CLI surfaces where a relation edge could or
should be authored:

- Audit `RELATION_RULES` against what the code actually accepts/emits/validates
  (table honesty) — add the missing PRD-parent row + the VT-1 golden-order update.
- Check every relation label has a coherent author/remove verb and that flag/arg
  shapes are consistent across `link`, `spec edit`, `spec interactions`,
  `spec req`, `review`, `rec`, `revision`, `concept-map`.
- Surface any remaining hand-edit-only or inconsistent edge and close it.

## Links

- Spawned from SL-153 design (§8 R2 follow-up).
