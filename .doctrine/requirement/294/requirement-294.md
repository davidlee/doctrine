# REQ-294: The orchestrator never widens a worker's source delta when integrating: it stages exact declared paths (never git add -A / commit -a), substitutes the checkout-import idiom when git apply patches corrupt or stat-proxy, and re-anchors onto a moved HEAD only on a per-path byte-identical disjointness proof.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
