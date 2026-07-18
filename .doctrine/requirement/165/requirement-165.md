# REQ-165: The manifest declares the projection policy, embedded and excluded from every projected set

## Statement

The manifest declares the *projection policy* — the minimal base fileset, the
materialize-on-demand surfaces, the dirs to create, the gitignore entries, and the
root markers. It is parsed from the embedded copy at runtime and is itself excluded
from every projected set.

## Rationale

The manifest ceases to be mere lay-down configuration and becomes the single
declared source of what projects and what materializes — so the installed fileset
is what the policy declares, not what the embed root happens to contain (ADR-019
D7). Forward-intent — `pending` until an implementation slice lands it.
