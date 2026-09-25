A shipped asset (install/, memory/, plugins/) may ground a claim on any address a client can resolve:

1. prose that stands without a reference - inline the fact;
2. a published logical address, reference/<name>.md, resolvable in every client via `doctrine library show`;
3. a shipped memory key, [[mem.<key>]], present in the shipped corpus;
4. a skill name - the shipped skills are invoked by name;
5. a CLI verb - the CLI is the source of truth for shapes;
6. an in-corpus relative path, admissible only where the target is installed beside the citing file (e.g. a skill linking to its own references/ sibling). A published install/ document has no file on disk in a client (ADR-019), so a relative link from it reaches nothing.

A repo-private entity id, or a repo-private source or spec path, is never permissible at any tier. The earlier blanket rejection of a "widened vocabulary" is struck: these are not a new citation syntax but the resolution seams a client already has.

Widened at the SL-267 design review, pass 2 (RV-391 F-5), on the reviewer's condition that the in-corpus relative path carries its boundary and the rejection sentence is removed rather than left standing.
