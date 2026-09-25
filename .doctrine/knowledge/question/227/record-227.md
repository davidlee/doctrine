The drift gate is deferred out of SL-267 (ISS-309 part 2). Two things must be
settled before it is built, and they are coupled.

**Which seam does the check ride?** Three candidates already exist:

- `IMP-163` — wire the SL-143 shipped-memory self-correction gate via the SL-147
  domain-map. The designed-for home, and the only one written for *shipped
  memory* specifically.
- `IMP-411` — makes `[[source]]` anchors verifiable at all, as the floor, then a
  conformance check over anchored shipped assets.
- `doctor` — already runs an unresolved-citation check (41 live warnings in this
  corpus). It detects a *dangling* citation, which is the opposite failure: the
  shipped-corpus defect is invisible precisely because it resolves — to the
  client's unrelated record. So `doctor` is the natural *host* and the wrong
  *predicate*.

**How is the duplicate rule settled?** The rule "shipped content must carry no
repo-local coupling" already exists twice, one sub-corpus each:
`mem.pattern.doctrine.shipped-skill-platform-independence` (glob `plugins/**`)
and `mem.pattern.doctrine.shipped-master-body-scrub` (memory bodies). Neither
reaches `install/` reference docs or shipped memory prose. A third parallel rule
would be the wrong answer — so the gate is the forcing function for unifying
them, or at least for one home with per-sub-corpus predicates.

**Evidence that a rule alone cannot work.** While SL-267's ledger enumerating
this defect class was being written, a concurrent in-flight edit introduced a
fresh `IMP-483` citation into `install/design-prompts/delegation.md`. A written
rule, a full defect list, and an audit in progress prevented nothing — see
observation `01a0d8f4-89dc-7832-b52b-ba4e9d62ef68`. The gate is not defence in
depth; on this evidence it is the only defence.

Owner: ISS-309 part 2, with IMP-163 and IMP-411 as the seams.
