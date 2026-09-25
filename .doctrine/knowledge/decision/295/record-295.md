For every ref a kind's own `show` accepts, `doctrine show <REF>` emits stdout
byte-identical to `doctrine <kind> show <REF>`. The router resolves the ref,
canonicalises it, and calls the owning kind's existing `run_show`; it owns no
renderer.
