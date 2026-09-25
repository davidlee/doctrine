For every prefixed ref a kind's own `show` accepts, `doctrine show <REF>` emits
stdout byte-identical to `doctrine <kind> show <REF>`. A bare id is a router
convenience, not part of that property: it resolves only when exactly one kind
holds the id, and otherwise refuses as ambiguous. The router resolves the ref,
canonicalises it, and calls the owning kind's existing `run_show`; it owns no
renderer.
